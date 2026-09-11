# REVIEW-20260906 Agents API 商业化缺口分析

- Status: active（P0 第 1 批实施中：GAP-01 已落地，GAP-02 进行中）
- Outcome: 基础运行面已达商业化水准；评测/计量/事件/版本治理四个域缺失，构成商业化阻塞
- Date: `2026-09-06`
- Owner: `agents-platform`
- Scope: app-api only（`crates/sdkwork-intelligence-agents-service/specs/openapi/agents-app-api.openapi.yaml`，113 个 operation）
- Benchmark: OpenAI Assistants / Responses / Batch / Evals / Usage / Webhooks、Anthropic Messages + Agent SDK、MCP、AWS Bedrock Agents（alias/version）、LangSmith、Coze / Dify 开放平台

## 1. 现状盘点（113 个操作按域分组）

| 域 | 操作数 | 操作 | 成熟度 |
|---|---|---|---|
| Agent 定义 | 6 | `agents.{list,create,retrieve,update,delete,restore}` | ✅ 对齐 Assistants CRUD |
| 运行时配置 | 12 | `providerBindings`×3、`modelConfigurations`×7、`modelSelections.apply`、`agentEngines.list` | ✅ 基本完整 |
| 结构化调用 | 1 | `agents.calls.create` | ⚠️ 仅同步入口 |
| 会话与对话 | 41 | `sessions`×6、`turns`×4、`turnInputQueueEntries`×8、`interactions`×7、`checkpoints`×5、`sessionRuntimeBindings`×6、`sessionItems`×3、`sessionUserStates`×3、`sessionActivitySummaries.list`、`itemFeedback`×2、`workspaceSessions/projectSessions`×5 | ✅ 行业最丰富面 |
| 任务调度 | 13 | `tasks`×8、`taskRuns`×4、`taskRunAttempts.list` | ✅ 完整，缺批量 |
| 组合槽 | 10 | `compositionSlots`×5、`projectCompositionSlots`×5 | ✅ 平台特有 |
| 工作区/项目 | 14 | `workspaces`×8、`projects`×6 | ✅ 完整 |
| 工具/MCP | 3 | `tools.{list,invoke}`、`mcpServers.list` | ⚠️ MCP 只读 |
| 预览/优化 | 2 | `previewResponses.create`、`promptOptimizations.create` | ✅ 平台亮点 |
| 资产 | 1 | `assets.list` | ⚠️ 只读、无上传检索 |
| 评测 | 0 | — | ❌ 缺失 |
| 计量/成本 | 0 | — | ❌ 缺失 |
| 事件/Webhook | 0 | — | ❌ 缺失 |
| Agent 版本治理 | 0 | — | ❌ 缺失 |
| 搜索 | 0 | — | ❌ 缺失 |
| 配额/限流 | 0 | — | ❌ 缺失 |

## 2. 缺口明细与对标

### P0 — 商业化阻塞（不补齐无法向付费客户交付）

#### GAP-01 Agent Call 异步化与结果拉取 — ✅ 已实施（2026-09-06）
- 对标：OpenAI Responses（后台模式 + retrieve）、Anthropic（long-running）、Bedrock（async invoke）
- 实施：
  - `POST /app/v3/api/ai/agents/{agentId}/calls` 扩展 `executionMode: sync|async`（async 返回 202 + queued 记录）
  - `GET /app/v3/api/ai/agents/{agentId}/calls`（`agents.calls.list`，keyset 分页 + status 过滤）
  - `GET /app/v3/api/ai/agents/{agentId}/calls/{executionId}`（`agents.calls.retrieve`）
  - 专表 `ai_agent_runtime_execution`（并入 greenfield baseline `database/ddl/baseline/postgres/0001_agents_baseline.sql`，pre-launch 不设 post-baseline 迁移），`(tenant_id, agent_id, execution_id)` 唯一；进程内执行器 + `recover_stale_agent_calls` 崩溃恢复
  - SDK：`listAgentCalls` / `getAgentCall` 导出；契约 1.1.0 + 校验器扩展；app-api 操作数 113 → 115
- 验证：`check-agent-call-contract` + `cargo test agent_call`（8 测）+ `generate-agents-api-docs` 再生成

#### GAP-02 用量计量与成本归因 — ✅ 已实施（2026-09-06）
- 对标：OpenAI `/v1/organization/usage`、LangSmith 计量、Coze/Dify 资源包
- 实施：
  - `GET /app/v3/api/ai/usage/summary`（`agents.usage.summary.retrieve`）：租户范围聚合 `turnCount` / `sessionCount` / token 总量；可选 `agentId` / `sessionId` / `modelId` 过滤 + `from`/`to` RFC 3339 时间窗（含头不含尾）
  - `GET /app/v3/api/ai/usage/records`（`agents.usage.records.list`）：turn 级明细，`(createdAt, id)` 降序 keyset 游标（opaque + scope fingerprint 防跨域重放）
  - 事实源：`ai_agent_turn.{input,output,cached}_tokens`，无独立台账；baseline 新增 `idx_ai_agent_turn_usage_timeline` / `idx_ai_agent_turn_usage_agent_timeline`
  - 分层：`usage.rs`（查询/记录/fingerprint）→ `AgentRepository` trait（默认 unsupported）→ InMemory + SqlAgentRepository（`SQL_SUMMARIZE_AGENT_USAGE` / `SQL_LIST_AGENT_USAGE_RECORDS`）→ application 用例 → HTTP → SDK `getUsageSummary` / `listUsageRecords`
  - 边界保持：计费/订单/配额归平台与网关（sdkwork-api-cloud-gateway），本仓库只提供计量事实面；app-api 操作数 115 → 117
- 验证：`check-agent-call-contract`（扩展 2c usage 段）+ `check-api-operation-patterns`（operationId action 对齐 retrieve/list）+ `check-pagination` / `check-api-response-envelope` / `check-route-path-collisions` + `cargo test usage`（4 测）+ `check-agent-sdk-workspace` + 文档再生成

#### GAP-03 Agent 版本与发布治理 — ✅ 已实施（2026-09-06）
- 对标：Bedrock（version + alias）、LangGraph（deployment revisions）、Coze（版本发布）
- 实施：
  - 专表 `ai_agent_version`（并入 baseline）：write-once 不可变快照（manifest + implementation 元数据），`version_number` 每 agent 单调递增，`(tenant, org, agent, version_id)` 与 `(…, version_number)` 双唯一
  - `POST /app/v3/api/ai/agents/{agentId}/versions`（`agents.versions.create`，201）+ `GET .../versions`（list，keyset by versionNumber 降序 + opaque scope-bound cursor）+ `GET .../versions/{versionId}`（retrieve）+ `POST .../{versionId}/activate`（`agents.versions.activate`）
  - 激活语义：每 agent 单一 `activated_at` 标记（单 SQL 原子切换），激活旧版本 = 回滚（不可变 manifest 写回活动定义）；版本行本身永不改写
  - 分层完整：domain `AgentVersionRecord` → ports（4 个默认 unsupported 方法）→ InMemory + Sql（`SQL_INSERT/SELECT/LIST/ACTIVATE_AGENT_VERSION`）→ application（授权 + 快照 + 回滚）→ HTTP → SDK `createAgentVersion` / `listAgentVersions` / `getAgentVersion` / `activateAgentVersion`；app-api 操作数 117 → 121
- 验证：`check-agent-call-contract`（2d 版本段）+ `check-api-operation-patterns` + `check-pagination` / `check-api-response-envelope` / `check-route-path-collisions` + `cargo test agent_version`（不可变性 + 激活回滚 2 测）+ `check-agent-sdk-workspace` + `db:validate` + 文档再生成

#### GAP-04 Webhooks / 事件订阅 — ✅ 已实施（2026-09-06）
- 对标：OpenAI Webhooks、Stripe 事件模型、Coze 开放平台回调
- 实施：
  - 专表 `ai_agent_webhook_subscription` + `ai_agent_webhook_delivery`（并入 baseline，table-registry 已登记）：订阅携带 HTTPS-only endpoint + 封闭事件词汇（`agent_call.completed/failed`、`task_run.completed/failed`、`interaction.requested`）+ HMAC 签名 secret；投递账本 `queued -> succeeded/failed` 记录 response_code / 有界 error_detail
  - `POST /app/v3/api/ai/webhooks`（create，201，secret 一次性回显）+ `GET .../webhooks`（list，offset 分页，secret 永不回显）+ `GET .../webhooks/{webhookId}`（retrieve，无 secret）+ `DELETE .../webhooks/{webhookId}`（delete，204 无 body）+ `POST .../webhooks/{webhookId}/test`（test：签名 payload + 10s 有界超时外呼 + 终态落账）
  - 签名：`Sdkwork-Signature: t=<unix-seconds>,v1=<hmac-sha256(secret, "<ts>.<payload>")>`（复用 `sdkwork-utils` 的 hmac + 常时比较；secret `whsec_` + 32 字节 OsRng 真随机）；URL 强制 HTTPS、≤2048 字符、无空白
  - 分层完整：`webhook.rs`（纯函数 + 4 单测）→ ports（6 个默认 unsupported 方法）→ InMemory + Sql（6 个 SQL 常量 + Row/adapter/转发）→ application（授权 + secret 卫生 + fail-closed 校验，3 测）→ HTTP（5 handlers）→ SDK `webhooks.ts` 5 函数；app-api 操作数 121 → 126
- 验证：`check-agent-call-contract`（2e webhooks 段 + 事件词汇 + SDK 导出）+ `check-api-operation-patterns`（`test` 动作已并入 COMMAND_ACTIONS 规范）+ `cargo test webhook`（订阅生命周期 + 签名投递 + URL/事件 fail-closed）+ `check-agent-sdk-workspace` + `db:validate` + 文档再生成

### P1 — 商业竞争力项（上线后 1-2 个迭代内）

#### GAP-05 评测（Evals）最小面
- 对标：OpenAI Evals、LangSmith、Dify 评测集
- 接口：`evalDatasets` CRUD、`evalRuns.create/list/retrieve`（对 agent 版本跑数据集 → 评分），评分器先内置（精确匹配/JSON Schema 校验/LLM 评分）
- 与 GAP-03 强耦合：评测对象是版本，不是活动定义
- 量级：L

#### GAP-06 MCP Server 管理面
- 对标：MCP registry、Claude Desktop connector 管理
- 现状：`mcpServers.list` 只读
- 接口：`mcpServers.{create,retrieve,update,delete,healthCheck}`；`tools.list` 增加 server 维度过滤
- 量级：M

#### GAP-07 会话搜索
- 对标：OpenAI conversations 检索、Coze 会话管理
- 接口：`GET /ai/sessions/search`（关键词 + 时间 + agent 过滤；语义检索可后置）
- 量级：S-M

#### GAP-08 配额与限流可视面
- 对标：OpenAI rate limits、Dify 用量配额
- 接口：`GET /ai/quotas`（当前配额与消耗）；超限返回 `429` + `ProblemDetail` 扩展字段
- 边界：限流执行点在网关；本仓库提供配额事实与消耗查询
- 量级：S-M（依赖 GAP-02 数据）

### P2 — 差异化补全（按客户需求排期)

| ID | 缺口 | 对标 | 量级 | 备注 |
|---|---|---|---|---|
| GAP-09 | 知识库/文件直连面 | OpenAI Files/Vector Stores、Coze 知识库 | L | 受 DRIVE_SPEC 约束，必须走 sdkwork-drive Uploader；先与 drive 团队对边界 |
| GAP-10 | Agent 导入导出（bundle） | LangGraph export、Coze clone | M | projects.import 已有先例 |
| GAP-11 | Tracing 导出（OTel） | LangSmith/Langfuse | M | 边界：kernel SPI；本仓库提供导出配置面 |
| GAP-12 | Subagent 编排注册面 | Anthropic subagents、LangGraph multi-agent | L | agent-as-tool 已有（深度 1），缺显式注册/图编排 |
| GAP-13 | 批量执行（Batch） | OpenAI Batch API | M | taskRuns 之上加批量提交 + 聚合结果 |
| GAP-14 | 第三方 API Key 面 | OpenAI API keys | M | 边界：网关管鉴权，本仓库管 key 与 agent 绑定元数据 |

## 3. 明确不做（边界声明）

- 计费/订单/支付：平台与网关职责，本仓库只暴露计量事实（GAP-02 已声明）。
- 模型训练/微调：非 Agents 域。
- 网关限流执行与 API Key 鉴权：sdkwork-api-cloud-gateway 职责。
- Kernel SPI 变更（turn 引擎、provider wire）：维持 sdkwork-kernel 所有权不变。

## 4. 实施约束（所有新增操作必须满足）

- OpenAPI 权威先行：新增 operation 先改 `agents-app-api.openapi.yaml`，`build.rs` 物化路由常量，`AGENT_APP_API_OPERATIONS` 注册并更新 inventory 断言，`generate-agents-api-docs.mjs` 同步 expectedCount。
- API_SPEC v3 封套：`SdkWorkApiResponse` + `SdkWorkResourceData`，错误走 HTTP 层 `ProblemDetail`，业务失败放 `item.status`。
- Wire DTO：camelCase、请求 `deny_unknown_fields`、i64 一律 string（`serde_int64`）。
- 每个域一个契约校验器扩展（参照 `scripts/check-agent-call-contract.mjs` 五层对齐模式）。
- 权限：新增操作声明 `x-sdkwork-permission`，默认收敛到 `ai.agents.use` 系。
- SDK：`@sdkwork/agents-app-sdk` 同步导出，禁止 raw HTTP。

## 5. 建议路线图

| 阶段 | 内容 | 出口判据 |
|---|---|---|
| 第 1 批 | GAP-01（calls 异步化）✅、GAP-02（usage 计量，进行中） | 商业客户可异步集成并核算成本 |
| 第 2 批 | GAP-03（版本治理）✅、GAP-04（webhooks）✅ | 可发布/回滚/事件驱动集成 |
| 第 3 批 | GAP-05~08 | 评测闭环 + MCP 生态 + 搜索 + 配额 |
| 第 4 批 | GAP-09~14 按客户需求 | 差异化补全 |
