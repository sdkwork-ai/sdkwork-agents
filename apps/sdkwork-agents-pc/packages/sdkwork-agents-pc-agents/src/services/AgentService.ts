import {
  getAgentsAppSdkClientWithSession,
  type SdkworkAgentsAppClient,
} from '@sdkwork/agents-pc-core/sdk/agentsAppSdkClient';
import type {
  AgentManagementProfile,
  AgentRecord,
  CreateAgentProviderBindingRequest,
  CreateAgentRequest,
  UpdateAgentRequest,
} from '@sdkwork/agents-pc-core/sdk';
import { syncAgentCompositionSlots } from './CompositionSlotSyncService';
import { createAgentBusinessId, createAgentExecutionId } from './businessIdentifiers';
import {
  DEFAULT_LIST_PAGE_SIZE,
  toOffsetPageInfo,
  type OffsetPageInfo,
} from '@sdkwork/agents-pc-core/sdk/pagination';

export interface AgentConfig {
  id?: string;
  name: string;
  description: string;
  avatar?: string;
  type: 'normal' | 'independent';
  systemPrompt?: string;
  knowledgeBaseIds?: string[];
  author?: string;
  users?: string;
  color?: string;
  iconName?: string;
  categoryId?: string;
  welcomeMessage?: string;
  debugMode?: boolean;
  jsonMode?: boolean;
  memoryEnabled?: boolean;
  model?: string;
  temperature?: number;
  suggestedPrompts?: string[];
  voiceIds?: string[];
  toolIds?: string[];
  skillIds?: string[];
  /** Bound external MCP server keys (synced to `slotKind: mcp` composition slots). */
  mcpServerKeys?: string[];
}

export interface AgentPreviewResponseRequest {
  config: AgentConfig;
  content: string;
  debugMode?: boolean;
  memoryEnabled?: boolean;
  model?: string;
  temperature?: number;
}

export interface AgentPreviewResponse {
  content: string;
  executionId: string;
  outputPayload?: unknown;
}

export interface AgentPromptOptimizeRequest {
  config: AgentConfig;
  prompt: string;
}

export interface AgentPromptOptimizeResult {
  executionId: string;
  optimizedPrompt: string;
  outputPayload?: unknown;
}

export interface AgentListPage {
  items: AgentConfig[];
  pageInfo: OffsetPageInfo;
}

export interface AgentService {
  createAgent(config: AgentConfig): Promise<AgentConfig>;
  updateAgent(id: string, config: Partial<AgentConfig>): Promise<AgentConfig>;
  publishAgent(id: string): Promise<void>;
  /** Interactive list page — one server page at a time (`PAGINATION_SPEC.md` §8). */
  listAgentsPage(params?: {
    page?: number;
    pageSize?: number;
    scope?: 'market' | 'mine';
    q?: string;
  }): Promise<AgentListPage>;
  getAgent(id: string): Promise<AgentConfig | null>;
  deleteAgent(id: string): Promise<void>;
  requestPreviewResponse(request: AgentPreviewResponseRequest): Promise<AgentPreviewResponse>;
  optimizePrompt(request: AgentPromptOptimizeRequest): Promise<AgentPromptOptimizeResult>;
}

export class AgentRuntimeExecutionUnavailableError extends Error {
  constructor(operation: string, executionId: string) {
    super(`Agent runtime execution did not return a ${operation}: ${executionId}.`);
    this.name = 'AgentRuntimeExecutionUnavailableError';
  }
}

type AgentCatalogScope = 'market' | 'mine';
type RecordLike = Record<string, unknown>;

function isRecord(value: unknown): value is RecordLike {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

const DEFAULT_AGENT_BINDING_ID = 'binding.manifest.default';
const DEFAULT_AGENT_PROVIDER_ID = 'provider.agent.manifest';
const DEFAULT_AGENT_CONFIGURATION_PROFILE_ID = 'profile.agent.manifest.default';
const DEFAULT_AGENT_PROVIDER_CAPABILITIES = ['model.chat', 'tool.invoke'] as const;
const AGENT_UI_CONFIG_CONSTRAINT_PREFIX = 'sdkwork.agent.pc.config:';

function asString(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined;
}

function asOptionalString(value: unknown): string | undefined {
  return typeof value === 'string' ? value : undefined;
}

function asStringArray(value: unknown): string[] | undefined {
  if (!Array.isArray(value)) {
    return undefined;
  }
  const strings = value
    .map((item) => asString(item))
    .filter((item): item is string => Boolean(item));
  return strings;
}

function cleanStringArray(value: unknown): string[] | undefined {
  return Array.isArray(value) ? asStringArray(value) ?? [] : undefined;
}

function asBoolean(value: unknown): boolean | undefined {
  return typeof value === 'boolean' ? value : undefined;
}

function asNumber(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
}

function asStringFrom(record: RecordLike, keys: string[]): string | undefined {
  for (const key of keys) {
    const value = asString(record[key]);
    if (value) {
      return value;
    }
  }
  return undefined;
}

function pickOutputString(output: unknown, keys: string[]): string | undefined {
  const direct = asString(output);
  if (direct) {
    return direct;
  }

  if (!isRecord(output)) {
    return undefined;
  }

  for (const key of keys) {
    const value = output[key];
    const normalized = asString(value);
    if (normalized) {
      return normalized;
    }
    if (isRecord(value)) {
      const nested = pickOutputString(value, keys);
      if (nested) {
        return nested;
      }
    }
  }

  return undefined;
}

function normalizeStandardToken(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/gu, '-')
    .replace(/^[._-]+|[._-]+$/gu, '')
    .replace(/\.{2,}/gu, '.')
    .slice(0, 96);
}

function normalizeStandardId(value: unknown, prefix: string): string | undefined {
  const raw = asString(value);
  if (!raw) {
    return undefined;
  }
  if (raw.startsWith(prefix)) {
    return raw;
  }
  const normalized = normalizeStandardToken(raw);
  return normalized ? `${prefix}${normalized}` : undefined;
}

function normalizeStandardIdArray(value: unknown, prefix: string): string[] | undefined {
  if (!Array.isArray(value)) {
    return undefined;
  }
  const ids = value
    .map((item) => normalizeStandardId(item, prefix))
    .filter((item): item is string => Boolean(item));
  return ids.length > 0 ? ids : [];
}

function stripStandardIdPrefix(value: string, prefix: string): string {
  return value.startsWith(prefix) ? value.slice(prefix.length) : value;
}

function stripStandardIdPrefixArray(value: unknown, prefix: string): string[] | undefined {
  const ids = asStringArray(value);
  return ids?.map((item) => stripStandardIdPrefix(item, prefix));
}

function normalizeModelForApi(value: unknown): string | undefined {
  const model = asString(value);
  if (!model) {
    return undefined;
  }
  if (model.startsWith('model.')) {
    return model;
  }
  const normalized = normalizeStandardToken(model);
  return normalized ? `model.${normalized}` : undefined;
}

function normalizeModelForRuntime(value: unknown): string | undefined {
  return normalizeModelForApi(value);
}

function normalizeModelForUi(value: unknown): string | undefined {
  const model = asString(value);
  if (!model) {
    return undefined;
  }
  return stripStandardIdPrefix(model, 'model.');
}

function requireAgentId(config: AgentConfig): string {
  const agentId = asString(config.id);
  if (agentId) {
    return agentId;
  }
  throw new Error('Agent id is required for backend agent runtime execution');
}

function buildAgentRuntimeInputPayload(
  config: AgentConfig,
  extra: Record<string, unknown>,
): Record<string, unknown> {
  return {
    agent: {
      id: config.id,
      name: config.name,
      description: config.description,
      avatar: config.avatar,
      type: config.type,
      systemPrompt: config.systemPrompt,
      knowledgeBaseIds: config.knowledgeBaseIds,
      categoryId: config.categoryId,
      welcomeMessage: config.welcomeMessage,
      debugMode: config.debugMode,
      jsonMode: config.jsonMode,
      memoryEnabled: config.memoryEnabled,
      model: normalizeModelForRuntime(config.model),
      temperature: config.temperature,
      suggestedPrompts: config.suggestedPrompts,
      voiceIds: config.voiceIds,
      toolIds: config.toolIds,
      skillIds: config.skillIds,
    },
    ...extra,
  };
}

function asAgentType(value: unknown): AgentConfig['type'] {
  return value === 'independent' ? 'independent' : 'normal';
}

function asOptionalAgentType(value: unknown): AgentConfig['type'] | undefined {
  if (value === 'independent' || value === 'normal') {
    return value;
  }
  return undefined;
}

function normalizeManifest(value: unknown): RecordLike {
  return isRecord(value) ? value : {};
}

function parseAgentUiConfigFromIntent(value: unknown): Partial<AgentConfig> {
  if (!isRecord(value) || !Array.isArray(value.constraints)) {
    return {};
  }
  const encoded = value.constraints
    .map((item) => asString(item))
    .find((item): item is string => Boolean(item?.startsWith(AGENT_UI_CONFIG_CONSTRAINT_PREFIX)));
  if (!encoded) {
    return {};
  }
  try {
    const parsed = JSON.parse(encoded.slice(AGENT_UI_CONFIG_CONSTRAINT_PREFIX.length)) as unknown;
    return isRecord(parsed) ? (parsed as Partial<AgentConfig>) : {};
  } catch {
    return {};
  }
}

function normalizeAgentManagementProfile(value: unknown): Partial<AgentConfig> {
  if (!isRecord(value)) {
    return {};
  }
  return {
    author: asString(value.author),
    avatar: asString(value.avatar),
    categoryId: asString(value.categoryId),
    color: asString(value.color),
    debugMode: asBoolean(value.debugMode),
    iconName: asString(value.iconName),
    jsonMode: asBoolean(value.jsonMode),
    knowledgeBaseIds: asStringArray(value.knowledgeBaseIds),
    memoryEnabled: asBoolean(value.memoryEnabled),
    model: normalizeModelForUi(value.model),
    skillIds: stripStandardIdPrefixArray(value.skillIds, 'skill.'),
    suggestedPrompts: asStringArray(value.suggestedPrompts),
    systemPrompt: asOptionalString(value.systemPrompt),
    temperature: asNumber(value.temperature),
    toolIds: stripStandardIdPrefixArray(value.toolIds, 'tool.'),
    type: asOptionalAgentType(value.type),
    users: asString(value.users),
    voiceIds: stripStandardIdPrefixArray(value.voiceIds, 'voice.'),
    welcomeMessage: asOptionalString(value.welcomeMessage),
  };
}

function definedAgentConfig(config: Partial<AgentConfig>): Partial<AgentConfig> {
  return Object.fromEntries(
    Object.entries(config).filter(([, value]) => value !== undefined),
  ) as Partial<AgentConfig>;
}

function normalizeAgentFromAgentRecord(record: AgentRecord): AgentConfig {
  const manifest = normalizeManifest(record.manifest);
  const legacyConfig = normalizeAgentManagementProfile(
    parseAgentUiConfigFromIntent(record.defaultCodeTaskIntent),
  );
  const profileConfig = normalizeAgentManagementProfile(record.managementProfile);
  const uiConfig = { ...definedAgentConfig(legacyConfig), ...definedAgentConfig(profileConfig) };
  const categoryId = asString(uiConfig.categoryId) ?? (Array.isArray(record.tags) ? record.tags[0] : undefined);
  return {
    id: record.agentId,
    name: record.displayName,
    description: record.description ?? '',
    avatar: asString(uiConfig.avatar) ?? '',
    type: asAgentType(uiConfig.type ?? record.implementationKind),
    systemPrompt: asOptionalString(uiConfig.systemPrompt) ?? asString(manifest.description),
    knowledgeBaseIds: asStringArray(uiConfig.knowledgeBaseIds),
    author: asString(uiConfig.author),
    users: asString(uiConfig.users),
    color: asString(uiConfig.color),
    iconName: asString(uiConfig.iconName),
    categoryId,
    welcomeMessage: asOptionalString(uiConfig.welcomeMessage),
    debugMode: asBoolean(uiConfig.debugMode),
    jsonMode: asBoolean(uiConfig.jsonMode),
    memoryEnabled: asBoolean(uiConfig.memoryEnabled),
    model: asString(uiConfig.model),
    temperature: asNumber(uiConfig.temperature),
    suggestedPrompts: asStringArray(uiConfig.suggestedPrompts),
    voiceIds: asStringArray(uiConfig.voiceIds),
    toolIds: asStringArray(uiConfig.toolIds),
    skillIds: asStringArray(uiConfig.skillIds),
  };
}

function buildAgentManifest(config: Partial<AgentConfig>, agentId: string): Record<string, unknown> {
  return {
    schema_version: '1.0.0',
    manifest_type: 'agent',
    agent_id: agentId,
    name: agentId,
    display_name: config.name ?? agentId,
    description: config.systemPrompt ?? config.description ?? '',
    version: '0.1.0',
    domain: 'intelligence',
    required_capabilities: [{ capability_id: 'model.chat' }],
    optional_capabilities: [
      { capability_id: 'tool.invoke' },
      { capability_id: 'knowledge.read' },
      { capability_id: 'memory.query' },
    ],
    event_families: ['agent.lifecycle'],
    owner: { name: config.author ?? 'sdkwork-agents-pc' },
    status: 'active',
  };
}

function buildAgentManagementProfile(config: Partial<AgentConfig>): AgentManagementProfile {
  const avatar = asString(config.avatar);
  return {
    author: config.author,
    ...(avatar ? { avatar } : {}),
    categoryId: config.categoryId,
    color: config.color,
    debugMode: config.debugMode,
    iconName: config.iconName,
    knowledgeBaseIds: cleanStringArray(config.knowledgeBaseIds),
    jsonMode: config.jsonMode,
    memoryEnabled: config.memoryEnabled,
    model: normalizeModelForApi(config.model),
    skillIds: normalizeStandardIdArray(config.skillIds, 'skill.'),
    suggestedPrompts: cleanStringArray(config.suggestedPrompts),
    systemPrompt: config.systemPrompt,
    temperature: config.temperature,
    toolIds: normalizeStandardIdArray(config.toolIds, 'tool.'),
    type: config.type,
    users: config.users,
    voiceIds: normalizeStandardIdArray(config.voiceIds, 'voice.'),
    welcomeMessage: config.welcomeMessage,
  };
}

function buildAgentDefaultCodeTaskIntent(config: Partial<AgentConfig>): Record<string, unknown> {
  const uiConfig = buildAgentManagementProfile(config);
  return {
    prompt: config.systemPrompt ?? config.description ?? 'Run the managed agent according to its manifest.',
    contextPaths: cleanStringArray(config.knowledgeBaseIds) ?? [],
    constraints: [
      `agent.type=${config.type ?? 'normal'}`,
      `${AGENT_UI_CONFIG_CONSTRAINT_PREFIX}${JSON.stringify(uiConfig)}`,
    ],
  };
}

function buildCreateAgentRequest(
  config: AgentConfig,
  agentId: string,
): CreateAgentRequest {
  const categoryId = asString(config.categoryId);
  return {
    agentId,
    code: agentId,
    displayName: config.name,
    description: config.description,
    manifest: buildAgentManifest(config, agentId),
    defaultCodeTaskIntent: buildAgentDefaultCodeTaskIntent(config),
    managementProfile: buildAgentManagementProfile(config),
    implementationProviderId: null,
    implementationKind: 'manifest-only',
    visibility: 'private',
    tags: categoryId ? [categoryId] : [],
    requestedAt: new Date().toISOString(),
  };
}

function buildUpdateAgentRequest(id: string, config: Partial<AgentConfig>): UpdateAgentRequest {
  const categoryId = asString(config.categoryId);
  return {
    ...(typeof config.name === 'string' ? { displayName: config.name } : {}),
    ...(typeof config.description === 'string' ? { description: config.description } : {}),
    ...(categoryId ? { tags: [categoryId] } : {}),
    manifest: buildAgentManifest({ ...config, id }, id),
    defaultCodeTaskIntent: buildAgentDefaultCodeTaskIntent(config),
    managementProfile: buildAgentManagementProfile(config),
    requestedAt: new Date().toISOString(),
  };
}

function mergeAgentConfig(
  currentConfig: AgentConfig,
  updatedConfig: Partial<AgentConfig>,
  id: string,
): AgentConfig {
  return {
    ...currentConfig,
    ...Object.fromEntries(
      Object.entries(updatedConfig).filter(([, value]) => value !== undefined),
    ),
    id,
  };
}

function buildDefaultProviderBindingRequest(): CreateAgentProviderBindingRequest {
  return {
    bindingId: DEFAULT_AGENT_BINDING_ID,
    providerId: DEFAULT_AGENT_PROVIDER_ID,
    implementationKind: 'manifest-only',
    configurationProfileId: DEFAULT_AGENT_CONFIGURATION_PROFILE_ID,
    capabilities: [...DEFAULT_AGENT_PROVIDER_CAPABILITIES],
    makeDefault: true,
    requestedAt: new Date().toISOString(),
  };
}

function isExistingProviderBindingConflict(error: unknown): boolean {
  if (!isRecord(error)) {
    return false;
  }

  const status = error.status ?? error.statusCode;
  if (status !== 409 && status !== '409') {
    return false;
  }

  const response = isRecord(error.response) ? error.response : undefined;
  const body = isRecord(error.body) ? error.body : undefined;
  const code =
    asString(error.code) ??
    (response ? asString(response.code) : undefined) ??
    (body ? asString(body.code) : undefined);
  const errorCategory =
    asString(error.errorCategory) ??
    (response ? asString(response.errorCategory) : undefined) ??
    (body ? asString(body.errorCategory) : undefined);
  const detail =
    asString(error.detail) ??
    asString(error.message) ??
    (response ? asString(response.detail) ?? asString(response.message) : undefined) ??
    (body ? asString(body.detail) ?? asString(body.message) : undefined);
  const normalizedText = [code, errorCategory, detail]
    .filter((item): item is string => Boolean(item))
    .join(' ')
    .toLowerCase();
  const referencesProviderBinding =
    normalizedText.includes('provider binding') ||
    normalizedText.includes('provider_binding') ||
    normalizedText.includes(DEFAULT_AGENT_BINDING_ID);

  if (referencesProviderBinding && (normalizedText.includes('already exists') || normalizedText.includes('cannot be created again'))) {
    return true;
  }

  return referencesProviderBinding && code === 'conflict';
}

function isAgentNotFoundError(error: unknown): boolean {
  if (!isRecord(error)) {
    return false;
  }

  const status = error.status ?? error.statusCode;
  if (status === 404 || status === '404') {
    return true;
  }

  const response = isRecord(error.response) ? error.response : undefined;
  const body = isRecord(error.body) ? error.body : undefined;
  const problem = isRecord(error.problem) ? error.problem : undefined;
  const code =
    asNumber(error.code) ??
    (response ? asNumber(response.code) : undefined) ??
    (body ? asNumber(body.code) : undefined) ??
    (problem ? asNumber(problem.code) : undefined);

  return code === 40401;
}

function asCatalogScope(record: RecordLike): AgentCatalogScope | undefined {
  const rawScope = asStringFrom(record, [
    'scope',
    'visibility',
    'catalogScope',
    'catalog_scope',
    'source',
    'ownerScope',
    'owner_scope',
  ])?.toLowerCase();

  if (rawScope) {
    if (['market', 'public', 'published', 'store', 'discover'].includes(rawScope)) {
      return 'market';
    }
    if (['mine', 'my', 'owned', 'private', 'workspace', 'draft'].includes(rawScope)) {
      return 'mine';
    }
  }

  if (record.isMine === true || record.mine === true || record.owned === true || record.isOwner === true) {
    return 'mine';
  }
  if (record.isMarket === true || record.market === true || record.public === true || record.published === true) {
    return 'market';
  }

  return undefined;
}


function collectAgentRecords(snapshot: unknown, keys: string[]): RecordLike[] {
  if (!isRecord(snapshot)) {
    return [];
  }

  const result: RecordLike[] = [];
  for (const key of keys) {
    const items = snapshot[key];
    if (!Array.isArray(items)) {
      continue;
    }
    for (const item of items) {
      if (isRecord(item)) {
        result.push(item);
      }
    }
  }
  return result;
}

function normalizeAgent(record: RecordLike): AgentConfig | undefined {
  const id = asStringFrom(record, ['id', 'agentId', 'agent_id', 'subjectId', 'subject_id']);
  const name = asStringFrom(record, ['name', 'displayName', 'display_name', 'title']);

  if (!id || !name) {
    return undefined;
  }

  const legacyConfig = normalizeAgentManagementProfile(
    parseAgentUiConfigFromIntent(record.defaultCodeTaskIntent),
  );
  const profileConfig = normalizeAgentManagementProfile(record.managementProfile);
  const uiConfig = { ...definedAgentConfig(legacyConfig), ...definedAgentConfig(profileConfig) };

  return {
    id,
    name,
    description: asStringFrom(record, ['description', 'desc', 'summary', 'intro']) ?? '',
    avatar: asString(uiConfig.avatar) ?? asStringFrom(record, ['avatar', 'avatarUrl', 'avatar_url', 'iconUrl', 'icon_url']) ?? '',
    type: asAgentType(uiConfig.type ?? record.type),
    systemPrompt: asOptionalString(uiConfig.systemPrompt) ?? asStringFrom(record, ['systemPrompt', 'system_prompt', 'prompt']),
    knowledgeBaseIds: asStringArray(uiConfig.knowledgeBaseIds),
    author: asString(uiConfig.author) ?? asStringFrom(record, ['author', 'authorName', 'author_name', 'ownerName', 'owner_name']),
    users: asString(uiConfig.users) ?? asStringFrom(record, ['users', 'usageCountText', 'usage_count_text', 'installCountText', 'install_count_text']),
    color: asString(uiConfig.color) ?? asStringFrom(record, ['color', 'colorClass', 'color_class']),
    iconName: asString(uiConfig.iconName) ?? asStringFrom(record, ['iconName', 'icon_name', 'icon']),
    categoryId: asString(uiConfig.categoryId) ?? asStringFrom(record, ['categoryId', 'category_id', 'category']),
    welcomeMessage: asOptionalString(uiConfig.welcomeMessage) ?? asStringFrom(record, ['welcomeMessage', 'welcome_message', 'greeting']),
    debugMode: asBoolean(uiConfig.debugMode),
    jsonMode: asBoolean(uiConfig.jsonMode),
    memoryEnabled: asBoolean(uiConfig.memoryEnabled),
    model: asString(uiConfig.model),
    temperature: asNumber(uiConfig.temperature),
    suggestedPrompts: asStringArray(uiConfig.suggestedPrompts),
    voiceIds: asStringArray(uiConfig.voiceIds),
    toolIds: asStringArray(uiConfig.toolIds),
    skillIds: asStringArray(uiConfig.skillIds),
  };
}

function uniqueAgents(records: RecordLike[]): AgentConfig[] {
  const agents = new Map<string, AgentConfig>();

  for (const record of records) {
    const agent = normalizeAgent(record);
    if (agent?.id) {
      agents.set(agent.id, agent);
    }
  }

  return Array.from(agents.values());
}

export function parseAgentCatalogSnapshot(snapshot: unknown, scope: AgentCatalogScope): AgentConfig[] {
  const scopedKeys =
    scope === 'market'
      ? ['marketAgents', 'market_agents', 'publicAgents', 'public_agents', 'publishedAgents', 'published_agents']
      : ['myAgents', 'my_agents', 'ownedAgents', 'owned_agents', 'workspaceAgents', 'workspace_agents'];

  const explicitRecords = collectAgentRecords(snapshot, scopedKeys);
  const sharedRecords = collectAgentRecords(snapshot, ['agents', 'agentCatalog', 'agent_catalog', 'catalog']);

  return uniqueAgents([
    ...sharedRecords.filter((record) => asCatalogScope(record) === scope),
    ...explicitRecords,
  ]);
}

class SdkworkAgentService implements AgentService {
  constructor(
    private readonly getAgentClient: () => SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession,
  ) {}

  async listAgentsPage(params: {
    page?: number;
    pageSize?: number;
    scope?: 'market' | 'mine';
    q?: string;
  } = {}): Promise<AgentListPage> {
    const page = params.page ?? 1;
    const pageSize = params.pageSize ?? DEFAULT_LIST_PAGE_SIZE;
    const trimmedQuery = params.q?.trim();
    const response = await this.getAgentClient().ai.agents.list({
      page,
      pageSize,
      ...(trimmedQuery ? { q: trimmedQuery } : {}),
      ...(params.scope ? { scope: params.scope } : {}),
    });
    return {
      items: response.items.map(normalizeAgentFromAgentRecord),
      pageInfo: toOffsetPageInfo(response.pageInfo),
    };
  }

  async getAgent(id: string): Promise<AgentConfig | null> {
    try {
      const response = await this.getAgentClient().ai.agents.retrieve(id);
      return normalizeAgentFromAgentRecord(response);
    } catch (error) {
      if (isAgentNotFoundError(error)) {
        return null;
      }
      throw error;
    }
  }

  async createAgent(config: AgentConfig): Promise<AgentConfig> {
    const agentId = asString(config.id) ?? createAgentBusinessId(config.name);
    const response = await this.getAgentClient().ai.agents.create(
      buildCreateAgentRequest(config, agentId),
    );
    const saved = normalizeAgentFromAgentRecord(response);
    await syncAgentCompositionSlots(this.getAgentClient(), saved.id ?? agentId, saved);
    return saved;
  }

  async updateAgent(id: string, config: Partial<AgentConfig>): Promise<AgentConfig> {
    const currentResponse = await this.getAgentClient().ai.agents.retrieve(id);
    const mergedConfig = mergeAgentConfig(
      normalizeAgentFromAgentRecord(currentResponse),
      config,
      id,
    );
    const response = await this.getAgentClient().ai.agents.update(
      id,
      buildUpdateAgentRequest(id, mergedConfig),
    );
    const saved = normalizeAgentFromAgentRecord(response);
    await syncAgentCompositionSlots(this.getAgentClient(), id, saved);
    return saved;
  }

  async publishAgent(id: string): Promise<void> {
    try {
      await this.getAgentClient().ai.agents.providerBindings.create(
        id,
        buildDefaultProviderBindingRequest(),
      );
    } catch (error) {
      if (!isExistingProviderBindingConflict(error)) {
        throw error;
      }
    }
  }

  async deleteAgent(id: string): Promise<void> {
    await this.getAgentClient().ai.agents.delete(id);
  }

  async requestPreviewResponse(request: AgentPreviewResponseRequest): Promise<AgentPreviewResponse> {
    const agentId = requireAgentId(request.config);
    const executionId = createAgentExecutionId('preview');
    const model = normalizeModelForRuntime(request.model ?? request.config.model);
    const response = await this.getAgentClient().ai.agents.previewResponses.create(
      agentId,
      {
        executionId,
        content: request.content,
        debugMode: request.debugMode ?? false,
        memoryEnabled: request.memoryEnabled ?? false,
        model,
        temperature: request.temperature,
        inputPayload: buildAgentRuntimeInputPayload(request.config, {
          content: request.content,
          debugMode: request.debugMode ?? false,
          memoryEnabled: request.memoryEnabled ?? false,
          model,
          temperature: request.temperature,
        }),
        requestedAt: new Date().toISOString(),
      },
    );
    const execution = response;
    const outputPayload = execution.outputPayload;
    const content = pickOutputString(outputPayload, [
      'reply',
      'content',
      'message',
      'text',
      'answer',
      'response',
      'output',
    ]);
    if (!content) {
      throw new AgentRuntimeExecutionUnavailableError('preview response', String(execution.executionId ?? ''));
    }
    return {
      content,
      executionId: String(execution.executionId ?? ''),
      outputPayload,
    };
  }

  async optimizePrompt(request: AgentPromptOptimizeRequest): Promise<AgentPromptOptimizeResult> {
    const agentId = requireAgentId(request.config);
    const executionId = createAgentExecutionId('prompt');
    const model = normalizeModelForRuntime(request.config.model);
    const response = await this.getAgentClient().ai.agents.promptOptimizations.create(
      agentId,
      {
        executionId,
        prompt: request.prompt,
        inputPayload: buildAgentRuntimeInputPayload(
          request.config,
          {
            ...(model ? { model } : {}),
            prompt: request.prompt,
          },
        ),
        requestedAt: new Date().toISOString(),
      },
    );
    const execution = response;
    const outputPayload = execution.outputPayload;
    const optimizedPrompt = pickOutputString(outputPayload, [
      'optimizedPrompt',
      'optimized_prompt',
      'prompt',
      'content',
      'text',
      'output',
    ]);
    if (!optimizedPrompt) {
      throw new AgentRuntimeExecutionUnavailableError('prompt optimization', String(execution.executionId ?? ''));
    }
    return {
      executionId: String(execution.executionId ?? ''),
      optimizedPrompt,
      outputPayload,
    };
  }
}

export function createSdkworkAgentService(
  getAgentClient?: () => SdkworkAgentsAppClient,
): AgentService {
  return new SdkworkAgentService(getAgentClient);
}

export let agentService = createSdkworkAgentService();

export function configureAgentService(getAgentClient?: () => SdkworkAgentsAppClient): AgentService {
  agentService = createSdkworkAgentService(getAgentClient);
  return agentService;
}
