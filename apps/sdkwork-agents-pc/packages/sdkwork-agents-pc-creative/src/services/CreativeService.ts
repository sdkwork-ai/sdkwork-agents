import type {
  GenerationMediaResult,
  GenerationRecord,
} from '@sdkwork/agents-pc-core/sdk/generationsService';
import {
  resolveCreativeCreationType,
  resolveCreativeGenerationDispatch,
} from '@sdkwork/agents-pc-core/sdk/creationTypes';
import { creativeModelCatalogService } from '@sdkwork/agents-pc-commons';
import { uuid } from '@sdkwork/utils';

import type { CreativeMessage, CreativeSession } from '../types';

async function loadGenerationsService() {
  const { agentsGenerationsService } = await import(
    '@sdkwork/agents-pc-core/sdk/generationsService'
  );
  return agentsGenerationsService;
}

/**
 * Display modality for a record. The previous implementation collapsed every
 * non-video modality onto `image`; the record already carries the modality the
 * command was dispatched with, so surface it as-is.
 */
function toMode(record: GenerationRecord): string {
  const modality = (record as { modality?: string }).modality;
  return modality?.trim() ? modality.trim() : 'image';
}

function toModelInfo(record: GenerationRecord): string {
  const provider = record.sourceProvider?.trim();
  if (provider) return provider;
  return `SDKWork ${toMode(record).replace(/_/gu, ' ')}`;
}

function toProgress(record: GenerationRecord): number {
  if (record.status === 'succeeded') return 100;
  if (record.status === 'running') return 67;
  return 29;
}

function toAssistantMessage(
  record: GenerationRecord,
  media: readonly GenerationMediaResult[],
  messageId = record.id,
): CreativeMessage {
  const mode = toMode(record);
  const imageUrls = media.filter((item) => item.kind === 'image').map((item) => item.url);
  const videoUrls = media.filter((item) => item.kind === 'video').map((item) => item.url);
  const audioUrls = media.filter((item) => item.kind === 'audio').map((item) => item.url);
  return {
    id: messageId,
    role: 'assistant',
    text: record.promptPreview || record.operationType,
    stage: record.status === 'succeeded' ? 'completed' : 'loading',
    progress: toProgress(record),
    mode,
    modelInfo: toModelInfo(record),
    imageUrl: imageUrls[0],
    imageUrls,
    videoUrl: videoUrls[0],
    videoUrls,
    audioUrl: audioUrls[0],
    audioUrls,
  };
}

async function toCreativeSession(record: GenerationRecord): Promise<CreativeSession> {
  const generationsService = await loadGenerationsService();
  const media = record.status === 'succeeded'
    ? await generationsService.listMediaResults(record.id)
    : [];
  const prompt = record.promptPreview || record.operationType;
  return {
    id: record.id,
    title: prompt.length > 18 ? `${prompt.slice(0, 18)}...` : prompt,
    messages: [
      {
        id: `${record.id}-prompt`,
        role: 'user',
        text: prompt,
      },
      toAssistantMessage(record, media),
    ],
  };
}

export class CreativeService {
  static async getSessions(): Promise<CreativeSession[]> {
    try {
      const generationsService = await loadGenerationsService();
      const page = await generationsService.listRecords({ pageSize: 50 });
      return Promise.all(page.items.map(toCreativeSession));
    } catch (error) {
      // The generations app service may be unavailable in hosted portal
      // environments (no route registered); the creative page degrades to an
      // empty session list instead of failing to load.
      console.error('Failed to load generation sessions.', error);
      return [];
    }
  }

  static async generateContent(
    prompt: string,
    mode: string,
    onUpdate: (message: CreativeMessage) => void,
    model?: string,
    options: { hasReferenceImages?: boolean } = {},
  ): Promise<CreativeMessage> {
    const generationsService = await loadGenerationsService();
    // The mode is preserved. It used to be collapsed by
    // `mode === 'video' ? 'video' : 'image'`, which silently replayed music,
    // voice, sound-effect, digital-human and action prompts as text-to-image.
    //
    // `agent` is deliberately left on its previous assisted-image behaviour: it
    // is not a generation modality, and re-pointing it is a separate product
    // decision from the modality-collapse fix.
    const dispatch = resolveCreativeGenerationDispatch(mode, options)
      ?? (mode === 'agent' ? resolveCreativeGenerationDispatch('image') : null);
    if (!dispatch) {
      throw new Error(
        `“${mode}” 不是内容生成类型，无法提交生成请求。`,
      );
    }
    const modality = dispatch.modality;
    // Resolve the selected model through the unified creative model catalog:
    // stale or deprecated ids fall back to the replacement/default model.
    const resolvedModel = model
      ? creativeModelCatalogService.resolveSelection(modality, model)
      : undefined;
    const resolvedModelLabel = resolvedModel
      ? creativeModelCatalogService.getDefinition(modality, resolvedModel)?.label ?? resolvedModel
      : undefined;
    const pendingMessage: CreativeMessage = {
      id: uuid(),
      role: 'assistant',
      text: prompt,
      stage: 'thinking',
      progress: 0,
      mode: resolveCreativeCreationType(mode).id,
      modelInfo: resolvedModelLabel ?? `SDKWork ${modality.replace(/_/gu, ' ')}`,
      imageUrls: [],
      videoUrls: [],
      audioUrls: [],
    };
    onUpdate(pendingMessage);

    const command = await generationsService.create({
      modality,
      operationType: dispatch.operationType,
      prompt,
      ...(resolvedModel ? { model: resolvedModel } : {}),
    });
    const record = await generationsService.waitForCompletion(command.generation, {
      onStatus(current) {
        onUpdate({
          ...pendingMessage,
          stage: current.status === 'queued' ? 'thinking' : 'loading',
          progress: toProgress(current),
          modelInfo: toModelInfo(current),
        });
      },
    });
    const media = await generationsService.listMediaResults(record.id);
    if (media.length === 0) {
      throw new Error('Generation completed without a renderable media result.');
    }
    const finalMessage = toAssistantMessage(record, media, pendingMessage.id);
    onUpdate(finalMessage);
    return finalMessage;
  }
}
