import {
  SDKWORK_MAINSTREAM_AGENT_MODEL_CATALOG,
} from '@sdkwork/models-pc-picker/mainstream-catalog';
import { createFallbackModel } from '@sdkwork/models-pc-picker/model-picker';
import type { MainstreamAgentModelCatalogEntry } from '@sdkwork/models-pc-picker/mainstream-catalog';
import type {
  ModelsPickerGroup,
  ModelsPickerOption,
} from '@sdkwork/models-pc-picker/model-picker-types';

/**
 * Chat header model picker catalog.
 *
 * Feeds the two-column ModelPicker (vendor column on the left, model list on
 * the right) from @sdkwork/models-pc-picker with the SDKWork Models mainstream
 * agent catalog. Option ids are the canonical model ids, which is exactly what
 * ChatService sends to the agent backend as the `model` value.
 */

/** Preferred chat default; falls back to the first catalog entry when absent. */
export const CHAT_DEFAULT_MODEL_ID = 'gemini-3.6-flash';

/** Persists the last chat-header model selection across refresh / revisit. */
export const CHAT_SELECTED_MODEL_STORAGE_KEY = 'chat_selected_model';

function agentChatSelectedModelStorageKey(agentId: string): string {
  return `${CHAT_SELECTED_MODEL_STORAGE_KEY}:${agentId}`;
}

function createChatModelPickerOption(
  entry: MainstreamAgentModelCatalogEntry,
): ModelsPickerOption {
  return {
    id: entry.modelId,
    catalogKey: entry.catalogKey,
    model: entry.modelId,
    name: entry.displayName,
    displayName: entry.displayName,
    desc: entry.description,
    description: entry.description,
    ver: entry.catalogVersion,
    versionLabel: entry.catalogVersion,
    vendorCode: entry.vendorCode,
    vendorName: entry.vendorName,
    modalities: [...(entry.modalities ?? entry.outputModalities)],
    inputModalities: [...entry.inputModalities],
    outputModalities: [...entry.outputModalities],
    capabilities: [],
    officialReferencePrices: [],
    priceAvailability: { status: 'unavailable' },
    providerCodes: [...entry.supportedProviderIds],
    supportsStreaming: true,
    supportsTools: entry.supportsTools,
    supportsJsonSchema: false,
  };
}

function buildChatModelPickerGroups(): ModelsPickerGroup[] {
  const groupsByVendorCode = new Map<string, ModelsPickerGroup>();
  for (const entry of SDKWORK_MAINSTREAM_AGENT_MODEL_CATALOG) {
    let group = groupsByVendorCode.get(entry.vendorCode);
    if (!group) {
      group = {
        id: entry.vendorCode,
        vendor: { code: entry.vendorCode, name: entry.vendorName },
        llms: [],
        images: [],
        videos: [],
        audios: [],
        music: [],
        sfx: [],
      };
      groupsByVendorCode.set(entry.vendorCode, group);
    }
    group.llms.push(createChatModelPickerOption(entry));
  }
  return Array.from(groupsByVendorCode.values());
}

export const chatModelPickerGroups: ModelsPickerGroup[] = buildChatModelPickerGroups();

/**
 * Builds a picker group for a custom LLM provider applied through
 * `model_configurations/apply` (engineId=rig). The custom model joins the
 * picker under its vendor code so the playground can chat through a
 * client-provided base URL + API key + model.
 */
export function createCustomProviderModelGroup(input: {
  vendorCode: string;
  vendorName: string;
  modelId: string;
}): ModelsPickerGroup {
  const option: ModelsPickerOption = {
    id: input.modelId,
    catalogKey: `custom.${input.vendorCode}.${input.modelId}`,
    model: input.modelId,
    name: input.modelId,
    displayName: input.modelId,
    desc: `Custom ${input.vendorName} provider`,
    description: `Custom ${input.vendorName} provider`,
    ver: 'custom',
    versionLabel: 'custom',
    vendorCode: input.vendorCode,
    vendorName: input.vendorName,
    modalities: ['text'],
    inputModalities: ['text'],
    outputModalities: ['text'],
    capabilities: [],
    officialReferencePrices: [],
    priceAvailability: { status: 'unavailable' },
    providerCodes: [input.vendorCode],
    supportsStreaming: false,
    supportsTools: false,
    supportsJsonSchema: false,
  };
  return {
    id: `custom.${input.vendorCode}`,
    vendor: { code: input.vendorCode, name: input.vendorName },
    llms: [option],
    images: [],
    videos: [],
    audios: [],
    music: [],
    sfx: [],
  };
}

export function resolveChatDefaultModelId(): string {
  if (chatModelPickerGroups.some((group) => group.llms.some((model) => model.id === CHAT_DEFAULT_MODEL_ID))) {
    return CHAT_DEFAULT_MODEL_ID;
  }
  return chatModelPickerGroups[0]?.llms[0]?.id ?? CHAT_DEFAULT_MODEL_ID;
}

export function isChatModelIdKnown(
  modelId: string,
  groups: ModelsPickerGroup[] = chatModelPickerGroups,
): boolean {
  const normalized = modelId.trim();
  if (!normalized) {
    return false;
  }
  return groups.some((group) => group.llms.some((model) => model.id === normalized));
}

export function readStoredChatSelectedModelId(): string | null {
  try {
    const stored = globalThis.localStorage?.getItem(CHAT_SELECTED_MODEL_STORAGE_KEY)?.trim();
    return stored || null;
  } catch {
    return null;
  }
}

export function persistChatSelectedModelId(modelId: string): void {
  const normalized = modelId.trim();
  if (!normalized) {
    return;
  }
  try {
    globalThis.localStorage?.setItem(CHAT_SELECTED_MODEL_STORAGE_KEY, normalized);
  } catch {
    // Ignore quota / private-mode failures; selection still works in-session.
  }
}

export function readStoredAgentChatSelectedModelId(agentId: string): string | null {
  const normalizedAgentId = agentId.trim();
  if (!normalizedAgentId) {
    return null;
  }
  try {
    const stored = globalThis.localStorage
      ?.getItem(agentChatSelectedModelStorageKey(normalizedAgentId))
      ?.trim();
    return stored || null;
  } catch {
    return null;
  }
}

export function persistAgentChatSelectedModelId(agentId: string | undefined, modelId: string): void {
  const normalized = modelId.trim();
  if (!normalized) {
    return;
  }
  const normalizedAgentId = agentId?.trim();
  if (!normalizedAgentId || normalizedAgentId === 'agent.chat.default') {
    persistChatSelectedModelId(normalized);
    return;
  }
  try {
    globalThis.localStorage?.setItem(
      agentChatSelectedModelStorageKey(normalizedAgentId),
      normalized,
    );
  } catch {
    // Ignore quota / private-mode failures; selection still works in-session.
  }
}

/**
 * Restores the model for an agent-scoped chat: per-agent storage, then the
 * agent's configured default, then the global chat default.
 */
export function resolveAgentChatSelectedModelId(
  agentId: string | undefined,
  agentDefaultModelId?: string,
  groups: ModelsPickerGroup[] = chatModelPickerGroups,
): string {
  const normalizedAgentId = agentId?.trim();
  if (normalizedAgentId && normalizedAgentId !== 'agent.chat.default') {
    const stored = readStoredAgentChatSelectedModelId(normalizedAgentId);
    if (stored && isChatModelIdKnown(stored, groups)) {
      return stored;
    }
    const configured = agentDefaultModelId?.trim();
    if (configured && isChatModelIdKnown(configured, groups)) {
      return configured;
    }
  }
  return resolveChatSelectedModelId(groups);
}

/**
 * Restores the last chat-header model when it is still present in the picker
 * catalog (so ModelPicker can echo the correct label). Falls back to the
 * preferred default when storage is empty, unavailable, or stale.
 */
export function resolveChatSelectedModelId(
  groups: ModelsPickerGroup[] = chatModelPickerGroups,
): string {
  const stored = readStoredChatSelectedModelId();
  if (stored && isChatModelIdKnown(stored, groups)) {
    return stored;
  }
  return resolveChatDefaultModelId();
}

export function createChatModelPickerFallback(): ModelsPickerOption {
  const defaultModelId = resolveChatDefaultModelId();
  const defaultModel = chatModelPickerGroups
    .flatMap((group) => group.llms)
    .find((model) => model.id === defaultModelId);
  return createFallbackModel(
    defaultModel?.name ?? defaultModelId,
    defaultModel?.desc ?? 'SDKWork Models mainstream agent model',
    '2026.08.03.1',
    'llms',
    defaultModel?.vendorName ?? 'SDKWork Models',
  );
}
