import type {
  CreativeModelCatalogSyncState,
  CreativeModelCatalogSyncStats,
  CreativeModelDefinition,
  CreativeModelLifecycle,
  CreativeModelModality,
  RemoteCreativeModelItem,
} from './creativeModelCatalogTypes';
import {
  CREATIVE_MODEL_RELEASE_STAGE,
  CREATIVE_MODEL_SHELF_STATE,
} from './creativeModelCatalogTypes';
import {
  REMOTE_MODALITY_QUERY,
  REMOTE_SYNC_MODALITIES,
  STATIC_CREATIVE_MODELS,
  STATIC_DEFAULT_MODEL_IDS,
} from './staticCreativeModelCatalog';

const REMOTE_LAYER_STORAGE_KEY = 'sdkwork_creative_model_catalog_remote_v1';
const SELECTED_MODELS_STORAGE_KEY = 'sdkwork_creative_selected_models_v1';
const SYNC_TTL_MS = 5 * 60 * 1000;
const REMOTE_PAGE_SIZE = 100;
const MAX_REMOTE_PAGES = 10;
const MAX_REPLACEMENT_HOPS = 5;

/**
 * Remote-managed layer persisted between sessions. Remote entries are the
 * authoritative subset for their modality: models that disappear from a
 * fresh snapshot are removed; models flagged deprecated/retired or carrying
 * a replacement are migrated.
 */
interface PersistedRemoteLayer {
  version: 1;
  syncedAt: string;
  entries: Partial<Record<CreativeModelModality, CreativeModelDefinition[]>>;
}

function isCreativeModelModality(value: string): value is CreativeModelModality {
  return value in STATIC_CREATIVE_MODELS;
}

function readJsonStorage<T>(key: string): T | null {
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) return null;
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

function writeJsonStorage(key: string, value: unknown): void {
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Storage may be unavailable (private mode, quota); the catalog still
    // works from memory with the static baseline.
  }
}

/** Map a wire release/shelf state pair to the client lifecycle. */
function toLifecycle(releaseStage: number | null, shelfState: number | null): CreativeModelLifecycle {
  if (shelfState != null && shelfState >= CREATIVE_MODEL_SHELF_STATE.ARCHIVED) return 'retired';
  if (releaseStage != null && releaseStage >= CREATIVE_MODEL_RELEASE_STAGE.RETIRED) return 'retired';
  if (releaseStage != null && releaseStage >= CREATIVE_MODEL_RELEASE_STAGE.DEPRECATED) return 'deprecated';
  if (releaseStage != null && releaseStage >= CREATIVE_MODEL_RELEASE_STAGE.PREVIEW) return 'preview';
  return 'active';
}

function toRemoteDefinition(item: RemoteCreativeModelItem, modality: CreativeModelModality, index: number): CreativeModelDefinition {
  return {
    id: item.model,
    modality,
    label: item.displayName || item.model,
    desc: item.description || item.capabilityIntro || `由 ${item.vendorCode} 提供的模型`,
    lifecycle: toLifecycle(item.releaseStage, item.shelfState),
    replacementModelId: item.replacementModel || null,
    source: 'remote',
    order: index,
  };
}

function definitionsEqual(left: CreativeModelDefinition, right: CreativeModelDefinition): boolean {
  return left.label === right.label
    && left.desc === right.desc
    && left.lifecycle === right.lifecycle
    && (left.replacementModelId ?? null) === (right.replacementModelId ?? null);
}

function isPickerVisible(definition: CreativeModelDefinition): boolean {
  return definition.lifecycle !== 'retired';
}

export class CreativeModelCatalogService {
  private version = 0;

  private listeners = new Set<() => void>();

  private remoteLayer: PersistedRemoteLayer = CreativeModelCatalogService.loadRemoteLayer();

  private selectedIds: Partial<Record<CreativeModelModality, string>> =
    readJsonStorage<Partial<Record<CreativeModelModality, string>>>(SELECTED_MODELS_STORAGE_KEY) ?? {};

  private syncState: CreativeModelCatalogSyncState = {
    status: 'idle',
    lastSyncedAt: this.remoteLayer.syncedAt ?? null,
    lastError: null,
    lastStats: null,
  };

  private syncInFlight: Promise<CreativeModelCatalogSyncStats> | null = null;
  private lastSyncStartedAt = 0;

  private mergedCache = new Map<CreativeModelModality, CreativeModelDefinition[]>();
  private visibleCache = new Map<CreativeModelModality, CreativeModelDefinition[]>();
  private indexCache = new Map<CreativeModelModality, Map<string, CreativeModelDefinition>>();

  private static loadRemoteLayer(): PersistedRemoteLayer {
    const persisted = readJsonStorage<PersistedRemoteLayer>(REMOTE_LAYER_STORAGE_KEY);
    if (persisted && persisted.version === 1 && persisted.entries) {
      return persisted;
    }
    return { version: 1, syncedAt: '', entries: {} };
  }

  // ---------------------------------------------------------------- listeners

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    this.version += 1;
    this.mergedCache.clear();
    this.visibleCache.clear();
    this.indexCache.clear();
    for (const listener of this.listeners) {
      listener();
    }
  }

  /** Monotonic store revision; lets useSyncExternalStore cache per revision. */
  getVersion(): number {
    return this.version;
  }

  // ------------------------------------------------------------------ queries

  /** All merged definitions for a modality, including retired entries. */
  getDefinitions(modality: CreativeModelModality): CreativeModelDefinition[] {
    const cached = this.mergedCache.get(modality);
    if (cached) return cached;
    const merged = this.mergeModality(modality);
    this.mergedCache.set(modality, merged);
    return merged;
  }

  /** Picker-visible definitions (retired entries are hidden). */
  getModels(modality: CreativeModelModality): CreativeModelDefinition[] {
    const cached = this.visibleCache.get(modality);
    if (cached) return cached;
    const visible = this.getDefinitions(modality).filter(isPickerVisible);
    this.visibleCache.set(modality, visible);
    return visible;
  }

  getDefinition(modality: CreativeModelModality, modelId: string | undefined): CreativeModelDefinition | null {
    if (!modelId) return null;
    return this.getIndex(modality).get(modelId) ?? null;
  }

  private getIndex(modality: CreativeModelModality): Map<string, CreativeModelDefinition> {
    const cached = this.indexCache.get(modality);
    if (cached) return cached;
    const index = new Map<string, CreativeModelDefinition>();
    for (const definition of this.getDefinitions(modality)) {
      index.set(definition.id, definition);
    }
    this.indexCache.set(modality, index);
    return index;
  }

  /**
   * Merge policy:
   * - static baseline entries always come first, in catalog order;
   * - a remote entry with the same id overrides the static lifecycle
   *   (deprecation/retirement) while keeping the curated presentation;
   * - remote-only entries are appended in server order;
   * - remote entries dropped by the server disappear automatically
   *   (the persisted layer is replaced on every sync snapshot).
   */
  private mergeModality(modality: CreativeModelModality): CreativeModelDefinition[] {
    const staticDefinitions = STATIC_CREATIVE_MODELS[modality].map((definition) => ({ ...definition }));
    const staticIndex = new Map(staticDefinitions.map((definition) => [definition.id, definition]));
    const remoteEntries = this.remoteLayer.entries[modality] ?? [];
    const merged: CreativeModelDefinition[] = staticDefinitions;
    let order = staticDefinitions.length;
    for (const remote of remoteEntries) {
      const existing = staticIndex.get(remote.id);
      if (existing) {
        existing.lifecycle = remote.lifecycle;
        existing.replacementModelId = remote.replacementModelId ?? null;
        continue;
      }
      merged.push({ ...remote, order });
      order += 1;
    }
    return merged;
  }

  // --------------------------------------------------------------- selection

  getDefaultModelId(modality: CreativeModelModality): string {
    return this.resolveSelection(modality, STATIC_DEFAULT_MODEL_IDS[modality]);
  }

  /**
   * Resolve a requested model id against the merged catalog:
   * unknown/stale ids fall back to the modality default; deprecated or
   * retired ids follow their replacement chain (bounded) before falling
   * back to the default.
   */
  resolveSelection(modality: CreativeModelModality, modelId: string | undefined): string {
    if (!modelId) return this.getDefaultModelId(modality);
    const index = this.getIndex(modality);
    let current = index.get(modelId);
    if (!current) return this.getDefaultModelId(modality);
    if (current.lifecycle === 'active' || current.lifecycle === 'preview') return current.id;
    const visited = new Set<string>([current.id]);
    let replacementId = current.replacementModelId ?? null;
    for (let hop = 0; hop < MAX_REPLACEMENT_HOPS && replacementId; hop += 1) {
      if (visited.has(replacementId)) break;
      visited.add(replacementId);
      const replacement = index.get(replacementId);
      if (!replacement) break;
      if (replacement.lifecycle === 'active' || replacement.lifecycle === 'preview') {
        return replacement.id;
      }
      replacementId = replacement.replacementModelId ?? null;
    }
    return this.getDefaultModelId(modality);
  }

  /**
   * Effective selection for a modality. Priority: persisted selection
   * (auto-migrated when deprecated) > parent-provided initial id (only when
   * it genuinely belongs to this modality, bootstrapped once) > default.
   */
  getSelectedModelId(modality: CreativeModelModality, initialModelId?: string): string {
    const persisted = this.selectedIds[modality];
    if (persisted) {
      const resolved = this.resolveSelection(modality, persisted);
      if (resolved !== persisted) {
        this.persistSelection(modality, resolved);
      }
      return resolved;
    }
    if (initialModelId) {
      const resolved = this.resolveSelection(modality, initialModelId);
      if (resolved === initialModelId) {
        this.persistSelection(modality, resolved);
        return resolved;
      }
    }
    return this.getDefaultModelId(modality);
  }

  selectModel(modality: CreativeModelModality, modelId: string): void {
    if (this.selectedIds[modality] === modelId) return;
    this.persistSelection(modality, modelId);
  }

  private persistSelection(modality: CreativeModelModality, modelId: string): void {
    this.selectedIds = { ...this.selectedIds, [modality]: modelId };
    writeJsonStorage(SELECTED_MODELS_STORAGE_KEY, this.selectedIds);
    this.notify();
  }

  getSyncState(): CreativeModelCatalogSyncState {
    return this.syncState;
  }

  // -------------------------------------------------------------------- sync

  /**
   * Pull the per-modality snapshots from the models app API and apply them
   * locally (add / update / remove / deprecate). TTL-guarded: repeated calls
   * within the sync window reuse the in-flight or last completed request.
   * Remote sync is best-effort — failures keep the last persisted layer and
   * never break the static baseline.
   */
  async refresh(options: { force?: boolean } = {}): Promise<CreativeModelCatalogSyncStats> {
    const now = Date.now();
    if (this.syncInFlight) return this.syncInFlight;
    if (!options.force && this.syncState.status === 'ready' && this.syncState.lastSyncedAt) {
      if (now - this.lastSyncStartedAt < SYNC_TTL_MS) {
        return this.syncState.lastStats ?? this.emptyStats();
      }
    }
    this.lastSyncStartedAt = now;
    this.syncState = { ...this.syncState, status: 'syncing', lastError: null };
    this.notify();
    this.syncInFlight = this.runSync().finally(() => {
      this.syncInFlight = null;
    });
    return this.syncInFlight;
  }

  private emptyStats(): CreativeModelCatalogSyncStats {
    return {
      modality: 'all',
      addedModelIds: [],
      updatedModelIds: [],
      removedModelIds: [],
      deprecatedModelIds: [],
      fetchedAt: new Date().toISOString(),
      catalogSize: 0,
    };
  }

  private async runSync(): Promise<CreativeModelCatalogSyncStats> {
    const stats: CreativeModelCatalogSyncStats = {
      modality: 'all',
      addedModelIds: [],
      updatedModelIds: [],
      removedModelIds: [],
      deprecatedModelIds: [],
      fetchedAt: new Date().toISOString(),
      catalogSize: 0,
    };
    const nextEntries: Partial<Record<CreativeModelModality, CreativeModelDefinition[]>> = {};
    try {
      for (const modality of REMOTE_SYNC_MODALITIES) {
        const snapshot = await this.fetchRemoteModality(modality);
        const applied = this.applyRemoteSnapshot(modality, snapshot);
        nextEntries[modality] = applied.entries;
        stats.addedModelIds.push(...applied.addedModelIds);
        stats.updatedModelIds.push(...applied.updatedModelIds);
        stats.removedModelIds.push(...applied.removedModelIds);
        stats.deprecatedModelIds.push(...applied.deprecatedModelIds);
      }
      this.remoteLayer = {
        version: 1,
        syncedAt: stats.fetchedAt,
        entries: nextEntries,
      };
      writeJsonStorage(REMOTE_LAYER_STORAGE_KEY, this.remoteLayer);
      stats.catalogSize = REMOTE_SYNC_MODALITIES.reduce(
        (total, modality) => total + this.getModels(modality).length,
        0,
      );
      this.syncState = {
        status: 'ready',
        lastSyncedAt: stats.fetchedAt,
        lastError: null,
        lastStats: stats,
      };
      this.notify();
      return stats;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.syncState = {
        ...this.syncState,
        status: 'error',
        lastError: message,
      };
      this.notify();
      throw error;
    }
  }

  private async fetchRemoteModality(modality: CreativeModelModality): Promise<RemoteCreativeModelItem[]> {
    const { getModelsAppSdkClientWithSession } = await import('@sdkwork/agents-pc-core/sdk/modelsAppSdkClient');
    const client = getModelsAppSdkClientWithSession();
    const modalities = [...REMOTE_MODALITY_QUERY[modality]];
    const items: RemoteCreativeModelItem[] = [];
    for (let page = 1; page <= MAX_REMOTE_PAGES; page += 1) {
      const response = await client.ai.models.list({ modalities, page, pageSize: REMOTE_PAGE_SIZE });
      items.push(...response.items);
      if (!response.pageInfo?.hasMore) break;
    }
    return items;
  }

  /**
   * Diff one modality snapshot against the current remote layer. The
   * snapshot is authoritative for remote-managed entries: missing ids are
   * removed, new ids are added, changed fields are updated, and entries that
   * became deprecated/retired are reported so callers can migrate stale
   * selections (resolveSelection handles the actual fallback).
   */
  private applyRemoteSnapshot(
    modality: CreativeModelModality,
    snapshot: RemoteCreativeModelItem[],
  ): {
    entries: CreativeModelDefinition[];
    addedModelIds: string[];
    updatedModelIds: string[];
    removedModelIds: string[];
    deprecatedModelIds: string[];
  } {
    const previous = this.remoteLayer.entries[modality] ?? [];
    const previousById = new Map(previous.map((definition) => [definition.id, definition]));
    const staticIds = new Set(STATIC_CREATIVE_MODELS[modality].map((definition) => definition.id));
    const snapshotById = new Map<string, CreativeModelDefinition>();

    const entries: CreativeModelDefinition[] = [];
    const addedModelIds: string[] = [];
    const updatedModelIds: string[] = [];
    const removedModelIds: string[] = [];
    const deprecatedModelIds: string[] = [];

    snapshot.forEach((item, index) => {
      // Hidden models are dropped from the catalog entirely; archived
      // entries stay as retired so stale selections can be migrated.
      if (item.shelfState === CREATIVE_MODEL_SHELF_STATE.HIDDEN) return;
      const definition = toRemoteDefinition(item, modality, entries.length);
      snapshotById.set(definition.id, definition);

      const before = previousById.get(definition.id);
      if (!before && !staticIds.has(definition.id) && definition.lifecycle !== 'retired') {
        addedModelIds.push(definition.id);
      }
      if (before && !definitionsEqual(before, definition)) {
        updatedModelIds.push(definition.id);
      }
      const becameDeprecated = (definition.lifecycle === 'deprecated' || definition.lifecycle === 'retired')
        && (!before || (before.lifecycle !== 'deprecated' && before.lifecycle !== 'retired'));
      if (becameDeprecated) deprecatedModelIds.push(definition.id);
      entries.push(definition);
    });

    for (const definition of previous) {
      if (!snapshotById.has(definition.id)) {
        removedModelIds.push(definition.id);
      }
    }

    return { entries, addedModelIds, updatedModelIds, removedModelIds, deprecatedModelIds };
  }

  /** Clear the remote layer and persisted selections (settings/test reset). */
  reset(): void {
    this.remoteLayer = { version: 1, syncedAt: '', entries: {} };
    this.selectedIds = {};
    try {
      window.localStorage.removeItem(REMOTE_LAYER_STORAGE_KEY);
      window.localStorage.removeItem(SELECTED_MODELS_STORAGE_KEY);
    } catch {
      // ignore storage failures
    }
    this.syncState = { status: 'idle', lastSyncedAt: null, lastError: null, lastStats: null };
    this.notify();
  }
}

export const creativeModelCatalogService = new CreativeModelCatalogService();

export { isCreativeModelModality };
