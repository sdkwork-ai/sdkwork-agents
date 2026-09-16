import { uuid } from "@sdkwork/utils";

import {
  readAppSdkSessionTokens,
  resolveAppSdkOrganizationId,
  resolveAppSdkTenantId,
} from "../session/session";
import { agentsDriveUploadService } from "./driveUploadService";
import {
  CREATIVE_DEFAULT_OPERATION_BY_MODALITY,
  type CreativeGenerationModality,
  type CreativeGenerationOperationType,
} from "./creationTypes";
import type {
  GenerationCommandResponse,
  GenerationModality,
  GenerationRecord,
  GenerationRecordPage,
  GenerationResult,
  GenerationResultPage,
  SdkworkGenerationsAppClient,
} from "./generationsAppSdkClient";

export type { GenerationRecord } from "./generationsAppSdkClient";
export type {
  CreativeGenerationModality,
  CreativeGenerationOperationType,
} from "./creationTypes";

export interface GenerationCommandInput {
  /**
   * Modality carried on the command. Every creation surface (image / video /
   * music / voice / sound effect / digital human / action) has its own modality;
   * an unknown modality is rejected instead of being downgraded to image.
   */
  modality: CreativeGenerationModality;
  /**
   * Explicit operation. Omitted operations resolve through
   * `CREATIVE_DEFAULT_OPERATION_BY_MODALITY` for the given modality.
   */
  operationType?: CreativeGenerationOperationType;
  prompt: string;
  model?: string;
  inputAssetIds?: readonly string[];
  parameters?: Record<string, unknown>;
}

/** Result kind a generation produced. `audio` covers music / voice / sound effect. */
export type GenerationMediaKind = "image" | "video" | "audio";

export interface GenerationMediaResult {
  generationResult: GenerationResult;
  kind: GenerationMediaKind;
  url: string;
}

export interface WaitForGenerationOptions {
  intervalMs?: number;
  maxAttempts?: number;
  onStatus?: (record: GenerationRecord) => void;
}

type Sleep = (milliseconds: number) => Promise<void>;
type GenerationsClientProvider = () => (
  SdkworkGenerationsAppClient | Promise<SdkworkGenerationsAppClient>
);

const DEFAULT_PAGE_SIZE = 50;
const DEFAULT_POLL_INTERVAL_MS = 1_500;
const DEFAULT_POLL_ATTEMPTS = 80;

function defaultSleep(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function loadGenerationsAppSdkClient(): Promise<SdkworkGenerationsAppClient> {
  const { getGenerationsAppSdkClientWithSession } = await import("./generationsAppSdkClient");
  return getGenerationsAppSdkClientWithSession();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value && typeof value === "object" && !Array.isArray(value));
}

function readString(record: Record<string, unknown>, keys: readonly string[]): string | undefined {
  for (const key of keys) {
    const value = record[key];
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return undefined;
}

function inferResultKind(result: GenerationResult): GenerationMediaKind | null {
  const snapshot = isRecord(result.resourceSnapshot) ? result.resourceSnapshot : {};
  const signal = [
    result.resultType,
    readString(snapshot, ["kind", "mediaType", "media_type", "contentType", "content_type"]),
  ].filter(Boolean).join(" ").toLowerCase();
  if (signal.includes("video")) return "video";
  if (signal.includes("image")) return "image";
  // Music / speech / sound-effect results carry `audio` (or the vendor-specific
  // `music` / `sfx` / `voice`) instead of an image or video media type. Without
  // this branch they were dropped from `listMediaResults`, which then made a
  // successful audio generation look like "completed without a renderable
  // media result".
  if (signal.includes("audio") || signal.includes("music") || signal.includes("sfx")
    || signal.includes("sound") || signal.includes("voice") || signal.includes("speech")) {
    return "audio";
  }
  return null;
}

function toGenerationRecordPage(value: unknown): GenerationRecordPage {
  if (!isRecord(value) || !Array.isArray(value.items)) {
    return { items: [] };
  }
  return {
    items: value.items as GenerationRecord[],
    ...(typeof value.nextCursor === "string" ? { nextCursor: value.nextCursor } : {}),
  };
}

function toGenerationResultPage(value: unknown): GenerationResultPage {
  if (!isRecord(value) || !Array.isArray(value.items)) {
    return { items: [] };
  }
  return {
    items: value.items as GenerationResult[],
    ...(typeof value.nextCursor === "string" ? { nextCursor: value.nextCursor } : {}),
  };
}

type GenerationsApi = SdkworkGenerationsAppClient["generations"];
type CreateGenerationCommandBody = Parameters<GenerationsApi["images"]["textToImage"]>[0];
type GenerationCommandParams = Parameters<GenerationsApi["images"]["textToImage"]>[1];

type GenerationOperationSender = (
  client: SdkworkGenerationsAppClient,
  body: CreateGenerationCommandBody,
  params: GenerationCommandParams,
) => Promise<GenerationCommandResponse>;

/**
 * Bridge for the two operations that are registered by the generations router
 * and declared in the app OpenAPI contract, but have no typed method in the
 * checked-in generated SDK (`avatar`, `motion_mimicry`). They go through the
 * same authenticated HTTP client — same base URL, same interceptors — so they
 * still reach the contract. Delete this once the SDK is regenerated and the
 * generated methods exist.
 */
async function sendUnlistedGenerationCommand(
  client: SdkworkGenerationsAppClient,
  path: string,
  body: CreateGenerationCommandBody,
  params: GenerationCommandParams,
): Promise<GenerationCommandResponse> {
  const { appApiPath } = await import("@sdkwork/generations-app-sdk");
  // `BaseHttpClient.request` lives outside this workspace (`@sdkwork/sdk-common`),
  // so its option type is described structurally here instead of imported.
  const http = client.http as unknown as {
    request<T>(path: string, options: {
      method: string;
      body?: unknown;
      contentType?: string;
      headers?: Record<string, string>;
      sdkworkUnwrapKind?: "item" | "page" | "command" | "data" | "void";
    }): Promise<T>;
  };
  return http.request<GenerationCommandResponse>(appApiPath(path), {
    method: "POST",
    body,
    contentType: "application/json",
    headers: { "Idempotency-Key": params.idempotencyKey },
    sdkworkUnwrapKind: "item",
  });
}

/**
 * One sender per operation. The previous implementation branched on four
 * operation types and forwarded everything else to `images.textToImage`, which
 * turned music / voice / sound-effect / digital-human / action commands into
 * silent image requests.
 */
const GENERATION_OPERATION_SENDERS: Record<CreativeGenerationOperationType, GenerationOperationSender> = {
  text_to_image: (client, body, params) => client.generations.images.textToImage(body, params),
  image_edit: (client, body, params) => client.generations.images.imageEdit(body, params),
  text_to_video: (client, body, params) => client.generations.videos.textToVideo(body, params),
  image_to_video: (client, body, params) => client.generations.videos.imageToVideo(body, params),
  video_extend: (client, body, params) => client.generations.videos.videoExtend(body, params),
  text_to_music: (client, body, params) => client.generations.music.textToMusic(body, params),
  lyrics_to_music: (client, body, params) => client.generations.music.lyricsToMusic(body, params),
  sound_effects: (client, body, params) => client.generations.soundEffects.create(body, params),
  speech: (client, body, params) => client.generations.voice.speech(body, params),
  transcription: (client, body, params) => client.generations.voice.transcription(body, params),
  translation: (client, body, params) => client.generations.voice.translation(body, params),
  avatar: (client, body, params) =>
    sendUnlistedGenerationCommand(client, "/generations/videos/avatar", body, params),
  motion_mimicry: (client, body, params) =>
    sendUnlistedGenerationCommand(client, "/generations/videos/motion_mimicry", body, params),
};

export class AgentsGenerationsService {
  constructor(
    private readonly getClient: GenerationsClientProvider = loadGenerationsAppSdkClient,
    private readonly resolveDrivePreviewUrl: (driveUri: string) => Promise<string> =
      (driveUri) => agentsDriveUploadService.resolvePreviewUrl(driveUri),
    private readonly sleep: Sleep = defaultSleep,
  ) {}

  async create(input: GenerationCommandInput): Promise<GenerationCommandResponse> {
    const prompt = input.prompt.trim();
    if (!prompt) {
      throw new Error("Generation prompt is required.");
    }
    const session = readAppSdkSessionTokens();
    const tenantId = resolveAppSdkTenantId(session);
    if (!tenantId) {
      throw new Error("An authenticated tenant context is required for generation.");
    }
    const body = {
      tenantId,
      ...(resolveAppSdkOrganizationId(session)
        ? { organizationId: resolveAppSdkOrganizationId(session) }
        : {}),
      prompt,
      ...(input.model?.trim() ? { model: input.model.trim() } : {}),
      ...(input.inputAssetIds?.length ? { inputAssetIds: [...input.inputAssetIds] } : {}),
      ...(input.parameters ? { parameters: input.parameters } : {}),
    };
    const params = { idempotencyKey: `agents-generation-${uuid()}` };
    const operationType = input.operationType
      ?? CREATIVE_DEFAULT_OPERATION_BY_MODALITY[input.modality];
    const sender = GENERATION_OPERATION_SENDERS[operationType];
    if (!sender) {
      throw new Error(
        `Unsupported generation operation "${String(operationType)}" for modality "${String(input.modality)}".`,
      );
    }
    const client = await this.getClient();
    return sender(client, body, params);
  }

  async listRecords(input: {
    cursor?: string;
    modality?: GenerationModality;
    pageSize?: number;
    q?: string;
  } = {}): Promise<GenerationRecordPage> {
    const client = await this.getClient();
    const page = await client.generations.list({
      ...(input.cursor ? { cursor: input.cursor } : {}),
      ...(input.modality ? { modality: input.modality } : {}),
      pageSize: input.pageSize ?? DEFAULT_PAGE_SIZE,
      ...(input.q?.trim() ? { q: input.q.trim() } : {}),
    });
    return toGenerationRecordPage(page);
  }

  async listResults(generationId: string): Promise<GenerationResultPage> {
    const client = await this.getClient();
    const page = await client.generations.results.list(generationId, {
      pageSize: DEFAULT_PAGE_SIZE,
    });
    return toGenerationResultPage(page);
  }

  async listMediaResults(generationId: string): Promise<GenerationMediaResult[]> {
    const page = await this.listResults(generationId);
    const media = await Promise.all(page.items.map(async (result) => {
      const kind = inferResultKind(result);
      if (!kind) return null;
      const snapshot = isRecord(result.resourceSnapshot) ? result.resourceSnapshot : {};
      const directUrl = readString(snapshot, ["url", "publicUrl", "public_url"]);
      if (directUrl) {
        return { generationResult: result, kind, url: directUrl };
      }
      const driveUri = result.driveUri
        ?? readString(snapshot, ["driveUri", "drive_uri", "uri"]);
      if (!driveUri) return null;
      const url = driveUri.startsWith("drive://")
        ? await this.resolveDrivePreviewUrl(driveUri)
        : driveUri;
      return { generationResult: result, kind, url };
    }));
    return media.filter((item): item is GenerationMediaResult => item !== null);
  }

  async waitForCompletion(
    initialRecord: GenerationRecord,
    options: WaitForGenerationOptions = {},
  ): Promise<GenerationRecord> {
    let record = initialRecord;
    const maxAttempts = options.maxAttempts ?? DEFAULT_POLL_ATTEMPTS;
    for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
      options.onStatus?.(record);
      if (record.status === "succeeded") return record;
      if (record.status === "failed" || record.status === "canceled") {
        throw new Error(`Generation ${record.status}.`);
      }
      await this.sleep(options.intervalMs ?? DEFAULT_POLL_INTERVAL_MS);
      const client = await this.getClient();
      record = await client.generations.retrieve(record.id);
    }
    throw new Error("Generation did not complete before the polling deadline.");
  }
}

export const agentsGenerationsService = new AgentsGenerationsService();
