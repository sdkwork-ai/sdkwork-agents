import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readText(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

test("runtime environment helpers fail closed for production-like profiles", () => {
  const source = readText("crates/sdkwork-agents-contract/src/lib.rs");
  const expectedProfiles = ["production", "prod", "staging", "stage", "live", "test"];
  const identifiersMatch = source.match(
    /const PRODUCTION_LIKE_ENV_IDENTIFIERS:[\s\S]*?&\[(?<items>[\s\S]*?)\];/,
  );

  assert.ok(
    identifiersMatch?.groups?.items,
    "sdkwork-agents-contract must declare PRODUCTION_LIKE_ENV_IDENTIFIERS",
  );

  const productionLikeProfiles = Array.from(
    identifiersMatch.groups.items.matchAll(/"([^"]+)"/g),
    (match) => match[1],
  );

  for (const profile of expectedProfiles) {
    assert.ok(
      productionLikeProfiles.includes(profile),
      `production-like profile '${profile}' must be gated`,
    );
  }

  assert.match(
    source,
    /pub fn agents_allow_contract_runtime_fallback\(\) -> bool\s*\{\s*agents_environment_is_explicitly_configured\(\) && !agents_is_production_like_environment\(\)\s*\}/,
    "contract runtime fallback must be disabled in production-like environments and when the environment is not configured",
  );
  assert.match(
    source,
    /pub fn agents_use_dev_inline_auth_resolver\(\) -> bool\s*\{\s*agents_environment_is_explicitly_configured\(\)\s*&& !agents_is_production_like_environment\(\)\s*&& agents_dev_auth_bypass_enabled\(\)\s*\}/,
    "dev inline auth resolver must require a configured non-production environment and an explicit bypass",
  );
  assert.match(
    source,
    /pub fn ensure_dev_auth_bypass_allowed\(\) -> Result<\(\), String>\s*\{[\s\S]*agents_dev_auth_bypass_enabled\(\) && agents_is_production_like_environment\(\)[\s\S]*return Err/,
    "dev auth bypass must fail closed when enabled in a production-like environment",
  );

  const deploymentEnvIndex = source.indexOf('std::env::var("SDKWORK_DEPLOYMENT_ENV")');
  const genericEnvIndex = source.indexOf('std::env::var("ENVIRONMENT")');
  assert.ok(
    deploymentEnvIndex >= 0 && genericEnvIndex > deploymentEnvIndex,
    "SDKWORK_DEPLOYMENT_ENV must be the first security gate environment authority",
  );
});

test("container profile entrypoint rejects lifecycle identity downgrades before process startup", () => {
  const source = readText("deployments/docker/agents-container-entrypoint.sh");
  const immutableIdentityFields = [
    "SDKWORK_AGENTS_PROFILE_ID",
    "SDKWORK_AGENTS_DEPLOYMENT_PROFILE",
    "SDKWORK_AGENTS_ENVIRONMENT",
    "SDKWORK_ENVIRONMENT",
  ];

  assert.match(
    source,
    /standalone\.development \| standalone\.test \| standalone\.staging \| standalone\.production[\s\S]*cloud\.development \| cloud\.test \| cloud\.staging \| cloud\.production/,
    "container startup must accept only declared topology profile identifiers",
  );
  assert.match(
    source,
    /profile_file="\/app\/etc\/topology\/\$\{profile_id\}\.env"/,
    "container startup must load the selected in-image topology profile",
  );
  assert.doesNotMatch(
    source,
    /SDKWORK_AGENTS_PROFILE_FILE:-/,
    "container startup must not allow an arbitrary profile-file environment override",
  );

  for (const field of immutableIdentityFields) {
    assert.match(
      source,
      new RegExp(`verify_existing_security_value ${field} `),
      `${field} must be compared with the selected profile before startup`,
    );
  }

  assert.match(
    source,
    /profile_agents_environment" \!= "\$expected_environment"[\s\S]*profile_environment" \!= "\$expected_environment"[\s\S]*selected profile lifecycle environment does not match/,
    "profile lifecycle settings must agree with the selected profile identifier",
  );
  assert.match(
    source,
    /test \| staging \| production\)[\s\S]*selected production-like profile must declare SDKWORK_CORS_ALLOWED_ORIGINS/,
    "production-like profiles must declare a non-empty CORS baseline",
  );
  assert.match(
    source,
    /effective_cors_allowed_origins=\$\(printenv SDKWORK_CORS_ALLOWED_ORIGINS\)[\s\S]*export SDKWORK_CORS_ALLOWED_ORIGINS="\$effective_cors_allowed_origins"/,
    "operator-provided exact CORS values must remain materializable after profile identity validation",
  );
});

test("production HTTP bootstrap uses IAM, Postgres, and runtime facade completion", () => {
  const source = readText(
    "crates/sdkwork-agents-kernel-bridge/src/agent_http_state.rs",
  );

  assert.match(
    source,
    /ensure_dev_auth_bypass_allowed\(\)[\s\S]*if agents_use_dev_inline_auth_resolver\(\)/,
    "HTTP state bootstrap must reject unsafe dev auth before considering the dev branch",
  );
  assert.match(
    source,
    /if agents_use_dev_inline_auth_resolver\(\)\s*\{[\s\S]*return dev_agent_http_state\([\s\S]*\);[\s\S]*\}\s*production_postgres_agent_http_state\([\s\S]*\)/,
    "production bootstrap must fall through to Postgres state when dev inline auth is unavailable",
  );
  assert.match(
    source,
    /AllowAllPolicyProvider::try_allow\("policy\.agents\.dev"\)[\s\S]*context\("build agents dev-only policy provider"\)\?/,
    "dev HTTP bootstrap must use the fallible dev-only policy constructor",
  );
  assert.match(
    source,
    /fn dev_agent_http_state\(\s*sandbox_port[\s\S]*AgentHttpState::with_turn_executor\([\s\S]*InMemoryAgentRepository::try_new[\s\S]*AllowAllPolicyProvider::try_allow\("policy\.agents\.dev"\)[\s\S]*build_routed_turn_executor\(/,
    "dev state must keep disposable storage and dev auth while executing turns through the route-aware runtime facade executor",
  );
  assert.match(
    source,
    /fn production_postgres_agent_http_state\(\s*sandbox_port[\s\S]*SqlAgentRepository::new[\s\S]*SqlAgentAuditSink::new_global[\s\S]*IamGatedPolicyProvider::default\(\)[\s\S]*build_routed_turn_executor\(/,
    "production state must use Postgres repository, Postgres audit, IAM policy, and RuntimeFacadeTurnExecutor",
  );
  assert.match(
    source,
    /AllowAllPolicyProvider` is only used for development scenarios/,
    "AllowAllPolicyProvider must remain documented as development-only",
  );
});

test("standalone gateway shutdown path does not panic on signal installation failure", () => {
  const source = readText(
    "crates/sdkwork-api-agents-standalone-gateway/src/shutdown.rs",
  );

  assert.doesNotMatch(
    source,
    /\.expect\(|panic!\(/,
    "shutdown signal handling must log installation failures instead of panicking",
  );
  assert.match(
    source,
    /tracing::warn!|tracing::error!/,
    "shutdown signal handling must emit an operator-visible log when signal setup fails",
  );
});

test("in-memory repository and audit locks recover from poisoned guards", () => {
  const source = readText(
    "crates/sdkwork-intelligence-agents-service/src/infrastructure.rs",
  );

  assert.doesNotMatch(
    source,
    /expect\("in-memory repository rwlock poisoned"\)|expect\("in-memory audit sink mutex poisoned"\)|expect\("agents metrics scrape state poisoned"\)/,
    "in-memory repository, audit, and metrics locks must recover and log instead of panicking",
  );
  assert.match(
    source,
    /trait RecoveringRwLock[\s\S]*fn recovering_read[\s\S]*fn recovering_write/,
    "in-memory repository lock recovery must stay centralized",
  );
  assert.match(
    source,
    /trait RecoveringMutex[\s\S]*fn recovering_lock/,
    "in-memory audit and metrics lock recovery must stay centralized",
  );
});

test("managed-store constructors propagate snowflake initialization errors", () => {
  const infrastructure = readText(
    "crates/sdkwork-intelligence-agents-service/src/infrastructure.rs",
  );
  const persistence = readText(
    "crates/sdkwork-intelligence-agents-service/src/persistence.rs",
  );
  const bridge = readText(
    "crates/sdkwork-agents-kernel-bridge/src/agent_http_state.rs",
  );

  assert.doesNotMatch(
    `${infrastructure}\n${persistence}`,
    /AgentBusinessIdGenerator::new_default\(\)\s*\.expect\(/,
    "managed-store constructors must propagate ID generator initialization errors instead of panicking",
  );
  assert.doesNotMatch(
    infrastructure,
    new RegExp(
      [
        "SECURITY " + "VIOLATION",
        "panic to " + "prevent " + "security",
        "will " + "panic to " + "prevent",
      ].join("|"),
    ),
    "production security validation must return explicit errors or fail closed instead of panicking",
  );
  assert.doesNotMatch(
    infrastructure,
    /expect\("message exists"\)/,
    "in-memory message updates must preserve not-found errors without panic-only assertions",
  );
  assert.match(
    infrastructure,
    /pub fn validate_production_security_config\(\) -> Result<\(\), String>/,
    "production security validation must be fallible",
  );
  assert.match(
    infrastructure,
    /pub fn try_allow\(provider_id: impl Into<String>\) -> Result<Self, String>/,
    "AllowAllPolicyProvider must expose a fallible dev-only constructor",
  );
  assert.match(
    infrastructure,
    /pub fn try_new\(\) -> KernelResult<Self>/,
    "in-memory repository must expose a fallible constructor for runtime bootstrap",
  );
  assert.match(
    bridge,
    /InMemoryAgentRepository::try_new\(\)[\s\S]*context\("build agents dev in-memory repository"\)\?/,
    "dev HTTP bootstrap must use the fallible in-memory repository constructor",
  );
  assert.match(
    persistence,
    /pub fn from_pool\(pool: BlockingPostgresPool\) -> KernelResult<Self>/,
    "postgres adapter from_pool must remain fallible when it builds the default ID generator",
  );
});

test("agent repository port does not compile incomplete persistence adapters", () => {
  const source = readText("crates/sdkwork-intelligence-agents-service/src/ports.rs");

  assert.doesNotMatch(
    source,
    /default stubs|backward compatibility with adapters|CapabilityMissing \{/,
    "AgentRepository must require production persistence methods instead of default stubs",
  );

  for (const methodName of [
    "insert_provider_binding",
    "update_provider_binding",
    "get_provider_binding",
    "insert_composition_slot",
    "update_composition_slot",
    "get_composition_slot",
    "insert_session",
    "update_session",
    "get_session",
    "list_sessions",
    "count_sessions",
    "append_session_item",
    "update_session_item",
    "get_session_item",
    "list_session_items",
    "count_session_items",
    "insert_turn_request",
    "update_turn_state",
    "complete_turn",
    "insert_interaction",
    "update_interaction",
    "get_interaction",
    "list_interactions",
    "count_interactions",
    "insert_task",
    "update_task",
    "get_task",
    "list_tasks",
  ]) {
    assert.match(
      source,
      new RegExp(`fn ${methodName}\\([\\s\\S]*?;`),
      `AgentRepository.${methodName} must be a required trait method`,
    );
  }
  assert.doesNotMatch(
    source,
    /fn count_tasks\(/,
    "high-volume Task lists must not reintroduce COUNT plus offset pagination",
  );
});

test("task worker is packaged and deployed as a hardened independently scalable process", () => {
  const dockerfile = readText("deployments/docker/Dockerfile");
  const deployment = readText("deployments/kubernetes/task-worker-deployment.yaml");
  const service = readText("deployments/kubernetes/task-worker-service.yaml");
  const pdb = readText("deployments/kubernetes/task-worker-pdb.yaml");
  const hpa = readText("deployments/kubernetes/task-worker-hpa.yaml");
  const topology = readText("specs/topology.spec.json");

  assert.match(dockerfile, /-p sdkwork-intelligence-agents-worker/);
  assert.match(dockerfile, /COPY sdkwork-kernel \.\/sdkwork-kernel/);
  assert.match(dockerfile, /COPY sdkwork-web-framework \.\/sdkwork-web-framework/);
  assert.match(dockerfile, /USER 65532:65532/);
  assert.match(deployment, /args:\s*\n\s*- sdkwork-intelligence-agents-worker/);
  assert.match(deployment, /fieldPath: metadata\.name/);
  assert.match(deployment, /fieldPath: metadata\.uid/);
  assert.match(deployment, /SDKWORK_AGENTS_TASK_WORKER_ID/);
  assert.match(deployment, /path: \/readyz/);
  assert.match(deployment, /path: \/healthz/);
  assert.match(deployment, /readOnlyRootFilesystem: true/);
  assert.match(deployment, /allowPrivilegeEscalation: false/);
  assert.match(deployment, /automountServiceAccountToken: false/);
  assert.match(service, /type: ClusterIP/);
  assert.match(pdb, /minAvailable: 1/);
  assert.match(hpa, /maxReplicas: 100/);
  assert.match(topology, /"id": "application\.task-worker"/);
  assert.doesNotMatch(
    `${deployment}\n${service}\n${pdb}\n${hpa}`,
    /lease[_-]?token|fencing[_-]?token/i,
    "deployment metadata must not expose scheduler lease or fencing material",
  );
});

test("SQL row adapter ports remain database-dialect neutral", () => {
  const source = readText(
    "crates/sdkwork-intelligence-agents-service/src/persistence.rs",
  );

  assert.match(source, /pub trait AgentRepositoryAdapter: Send \+ Sync/);
  assert.match(source, /pub trait AgentAuditAdapter: Send \+ Sync/);
  assert.match(source, /pub struct SqlAgentRepository<A>/);
  assert.match(source, /pub struct SqlAgentAuditSink<A>/);
  assert.doesNotMatch(
    source,
    /PostgresAgentRepositoryAdapter|PostgresAuditAdapter/,
    "shared row adapter ports must not encode one SQL dialect in their names",
  );
});

test("postgres read failures propagate instead of masquerading as empty or missing data", () => {
  const source = readText(
    "crates/sdkwork-intelligence-agents-service/src/persistence.rs",
  );

  assert.doesNotMatch(
    source,
    /database (?:list|count) query failed; returning (?:empty result|0)/,
    "postgres read failures must reach HTTP problem mapping instead of returning successful empty pages",
  );
  assert.doesNotMatch(
    source,
    /dropping malformed postgres row/,
    "malformed persisted rows must fail the page instead of silently changing its contents",
  );
  assert.doesNotMatch(
    source,
    /\.ok\(\)\s*\.flatten\(\)/,
    "postgres point reads must not turn query failures into missing resources",
  );
  for (const rowType of [
    "AgentBusinessRow",
    "AgentProviderBindingRow",
    "AgentCompositionSlotRow",
    "AgentSessionRow",
    "AgentSessionItemRow",
    "AgentInteractionRow",
    "AgentTaskRow",
  ]) {
    assert.match(
      source,
      new RegExp(`KernelResult<Option<${rowType}>>`),
      `postgres ${rowType} point reads must expose repository failures`,
    );
  }
  assert.match(
    source,
    /fn list_session_item_rows\([\s\S]*?\) -> KernelResult<Vec<AgentSessionItemRow>>/,
    "high-volume Session Item reads must expose repository failures",
  );
});

test("blocking service and provider work use bounded observable capacity", () => {
  const http = readText("crates/sdkwork-intelligence-agents-service/src/http.rs");
  const turnRuntime = readText(
    "crates/sdkwork-intelligence-agents-service/src/turn_runtime.rs",
  );
  const metrics = readText(
    "crates/sdkwork-intelligence-agents-service/src/infrastructure.rs",
  );

  assert.match(http, /SERVICE_WORKER_LIMIT[\s\S]*try_acquire_owned/);
  assert.match(turnRuntime, /PROVIDER_WORKER_LIMIT[\s\S]*try_acquire_owned/);
  assert.match(metrics, /sdkwork_agents_service_worker_rejections_total/);
  assert.match(metrics, /sdkwork_agents_provider_worker_rejections_total/);
});

test("route manifest build script returns explicit errors instead of panicking", () => {
  const source = readText("crates/sdkwork-routes-agents-http-shared/build.rs");

  assert.match(
    source,
    /fn main\(\) -> Result<\(\), Box<dyn std::error::Error>>/,
    "route manifest build script must expose a fallible main for build-critical errors",
  );
  assert.doesNotMatch(
    source,
    /panic!\(|\.expect\(/,
    "route manifest build script must propagate environment, OpenAPI parsing, and file write errors instead of panicking",
  );
  assert.match(
    source,
    /read_to_string\(path\)[\s\S]*failed to read OpenAPI authority/,
    "OpenAPI authority read errors must name the failed authority path",
  );
  assert.match(
    source,
    /serde_yaml::from_str\(&yaml\)[\s\S]*failed to parse OpenAPI authority/,
    "OpenAPI authority parse errors must name the failed authority path",
  );
}
);

test("operator docs expose production security smoke criteria", () => {
  const smokeRunbook = readText("docs/runbooks/smoke-test.md");
  const prd = readText("docs/product/prd/PRD.md");
  const techArchitecture = readText("docs/architecture/tech/TECH_ARCHITECTURE.md");

  assert.match(
    smokeRunbook,
    /SDKWORK_AGENTS_DEV_AUTH_BYPASS=true` must be `false` in staging\/production/,
    "smoke runbook must require dev auth bypass to be false in staging and production",
  );
  assert.match(
    smokeRunbook,
    /runtimeMode`[\s\S]{0,80}contract stub/,
    "smoke runbook must reject contract stub runtime mode for chat",
  );
  assert.match(
    prd,
    /Release Gates[\s\S]*check:production-security/,
    "PRD production gates must include the production-security check",
  );
  assert.match(
    techArchitecture,
    /check:production-security/,
    "technical architecture verification must include the production-security check",
  );
});
