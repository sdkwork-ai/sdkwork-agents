import { useCallback, useEffect, useMemo, useSyncExternalStore } from 'react';

import type {
  CreativeModelCatalogSyncState,
  CreativeModelDefinition,
  CreativeModelModality,
} from './creativeModelCatalogTypes';
import { creativeModelCatalogService } from './creativeModelCatalogService';

export interface CreativeModelCatalog {
  /** Picker-visible merged definitions (static baseline + remote layer). */
  models: CreativeModelDefinition[];
  defaultModelId: string;
  /** Effective selected model id (persisted, migrated, or default). */
  selectedModelId: string;
  selectModel: (modelId: string) => void;
  /** Resolve an arbitrary id (e.g. a canvas node's model) with fallbacks. */
  resolveModelId: (modelId: string | undefined) => string;
  getDefinition: (modelId: string | undefined) => CreativeModelDefinition | null;
  syncState: CreativeModelCatalogSyncState;
  refresh: (options?: { force?: boolean }) => Promise<unknown>;
}

interface CatalogSnapshot {
  models: CreativeModelDefinition[];
  defaultModelId: string;
  selectedModelId: string;
  syncState: CreativeModelCatalogSyncState;
  version: number;
}

const snapshotCache = new Map<string, CatalogSnapshot>();

function snapshotCacheKey(modality: CreativeModelModality, initialModelId: string | undefined): string {
  return `${modality}::${initialModelId ?? ''}`;
}

/**
 * React binding for the unified creative model catalog.
 *
 * One hook per creative modality; all instances share the singleton service
 * store, so a remote sync or selection change re-renders every consumer.
 * The first remote sync happens in the background and never blocks UI —
 * the static sdkwork-models baseline renders immediately.
 */
export function useCreativeModelCatalog(
  modality: CreativeModelModality,
  options: { initialModelId?: string } = {},
): CreativeModelCatalog {
  const { initialModelId } = options;

  const subscribe = useCallback(
    (listener: () => void) => creativeModelCatalogService.subscribe(listener),
    [],
  );

  const getSnapshot = useCallback(() => {
    const version = creativeModelCatalogService.getVersion();
    const key = snapshotCacheKey(modality, initialModelId);
    const cached = snapshotCache.get(key);
    if (cached && cached.version === version) return cached;
    const snapshot: CatalogSnapshot = {
      models: creativeModelCatalogService.getModels(modality),
      defaultModelId: creativeModelCatalogService.getDefaultModelId(modality),
      selectedModelId: creativeModelCatalogService.getSelectedModelId(modality, initialModelId),
      syncState: creativeModelCatalogService.getSyncState(),
      version,
    };
    snapshotCache.set(key, snapshot);
    return snapshot;
  }, [modality, initialModelId]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  // Background sync; the service TTL-guards repeated calls. Failures are
  // intentionally swallowed here (syncState.lastError surfaces the cause).
  useEffect(() => {
    creativeModelCatalogService.refresh().catch(() => undefined);
  }, []);

  const selectModel = useCallback(
    (modelId: string) => {
      creativeModelCatalogService.selectModel(modality, modelId);
    },
    [modality],
  );

  const resolveModelId = useCallback(
    (modelId: string | undefined) => creativeModelCatalogService.resolveSelection(modality, modelId),
    [modality],
  );

  const getDefinition = useCallback(
    (modelId: string | undefined) => creativeModelCatalogService.getDefinition(modality, modelId),
    [modality],
  );

  const refresh = useCallback(
    (refreshOptions?: { force?: boolean }) => creativeModelCatalogService.refresh(refreshOptions),
    [],
  );

  return useMemo(
    () => ({
      models: snapshot.models,
      defaultModelId: snapshot.defaultModelId,
      selectedModelId: snapshot.selectedModelId,
      selectModel,
      resolveModelId,
      getDefinition,
      syncState: snapshot.syncState,
      refresh,
    }),
    [snapshot, selectModel, resolveModelId, getDefinition, refresh],
  );
}
