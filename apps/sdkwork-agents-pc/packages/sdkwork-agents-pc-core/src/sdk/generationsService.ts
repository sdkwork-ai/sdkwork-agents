import { uuid } from "@sdkwork/utils";

import {
  readAppSdkSessionTokens,
  resolveAppSdkOrganizationId,
  resolveAppSdkTenantId,
} from "../session/session";
import { agentsDriveUploadService } from "./driveUploadService";
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

export interface GenerationCommandInput {
  modality: "image" | "video";
  operationType?: "image_edit" | "image_to_video" | "text_to_image" | "text_to_video";
  prompt: string;
  model?: string;
  inputAssetIds?: readonly string[];
  parameters?: Record<string, unknown>;
}

export interface GenerationMediaResult {
  generationResult: GenerationResult;
  kind: "image" | "video";
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

function inferResultKind(result: GenerationResult): "image" | "video" | null {
  const snapshot = isRecord(result.resourceSnapshot) ? result.resourceSnapshot : {};
  const signal = [
    result.resultType,
    readString(snapshot, ["kind", "mediaType", "media_type", "contentType", "content_type"]),
  ].filter(Boolean).join(" ").toLowerCase();
  if (signal.includes("video")) return "video";
  if (signal.includes("image")) return "image";
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
      ?? (input.modality === "video" ? "text_to_video" : "text_to_image");
    const generations = (await this.getClient()).generations;

    if (operationType === "image_edit") {
      return generations.images.imageEdit(body, params);
    }
    if (operationType === "image_to_video") {
      return generations.videos.imageToVideo(body, params);
    }
    if (operationType === "text_to_video") {
      return generations.videos.textToVideo(body, params);
    }
    return generations.images.textToImage(body, params);
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
