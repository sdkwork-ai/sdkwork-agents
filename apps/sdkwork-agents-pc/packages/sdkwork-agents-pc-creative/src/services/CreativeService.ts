import type {
  GenerationMediaResult,
  GenerationRecord,
} from '@sdkwork/agents-pc-core/sdk/generationsService';
import { uuid } from '@sdkwork/utils';

import type { CreativeMessage, CreativeSession } from '../types';

async function loadGenerationsService() {
  const { agentsGenerationsService } = await import(
    '@sdkwork/agents-pc-core/sdk/generationsService'
  );
  return agentsGenerationsService;
}

function toMode(record: GenerationRecord): 'image' | 'video' {
  return record.modality === 'video' ? 'video' : 'image';
}

function toModelInfo(record: GenerationRecord): string {
  return record.sourceProvider?.trim()
    || (record.modality === 'video' ? 'SDKWork Video' : 'SDKWork Image');
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
  ): Promise<CreativeMessage> {
    const generationsService = await loadGenerationsService();
    const normalizedMode = mode === 'video' ? 'video' : 'image';
    const pendingMessage: CreativeMessage = {
      id: uuid(),
      role: 'assistant',
      text: prompt,
      stage: 'thinking',
      progress: 0,
      mode: normalizedMode,
      modelInfo: normalizedMode === 'video' ? 'SDKWork Video' : 'SDKWork Image',
      imageUrls: [],
      videoUrls: [],
    };
    onUpdate(pendingMessage);

    const command = await generationsService.create({
      modality: normalizedMode,
      operationType: normalizedMode === 'video' ? 'text_to_video' : 'text_to_image',
      prompt,
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
