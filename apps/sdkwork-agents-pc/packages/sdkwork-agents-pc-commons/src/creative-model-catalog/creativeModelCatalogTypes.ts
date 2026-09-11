/**
 * Unified creative model catalog contracts shared by the image generation,
 * inspiration and video generation creation modes.
 *
 * Data layering (high cohesion, low coupling):
 * - `static`: baseline catalog seeded from the `sdkwork-models` repository
 *   static catalog. Always available, works offline.
 * - `remote`: authoritative snapshot fetched from the models app API
 *   (`GET /app/v3/api/ai/models?modalities=...`). Remote entries are added,
 *   updated, removed and deprecated through the sync service; static entries
 *   stay as the offline fallback unless the remote explicitly deprecates them.
 */

export type CreativeModelModality =
  | 'image'
  | 'video'
  | 'music'
  | 'voice'
  | 'digital_human'
  | 'action';

export type CreativeModelLifecycle = 'active' | 'preview' | 'deprecated' | 'retired';

export type CreativeModelSource = 'static' | 'remote';

export interface CreativeModelDefinition {
  /** Stable model id submitted with generation commands. */
  id: string;
  modality: CreativeModelModality;
  label: string;
  desc: string;
  /** Member-exclusive highlight marker. */
  spark?: boolean;
  /** Cover image URL for avatar-style pickers. */
  image?: string;
  /** Big badge text for version-style pickers. */
  badge?: string;
  subBadge?: string;
  isNew?: boolean;
  lifecycle: CreativeModelLifecycle;
  /** Preferred replacement when this model is deprecated or retired. */
  replacementModelId?: string | null;
  source: CreativeModelSource;
  /** Display order hint; static entries keep catalog order, remote entries follow server order. */
  order: number;
}

export interface CreativeModelCatalogSyncStats {
  modality: CreativeModelModality | 'all';
  addedModelIds: string[];
  updatedModelIds: string[];
  removedModelIds: string[];
  deprecatedModelIds: string[];
  fetchedAt: string;
  /** Total picker-visible definitions after the sync was applied. */
  catalogSize: number;
}

export type CreativeModelCatalogSyncStatus = 'idle' | 'syncing' | 'ready' | 'error';

export interface CreativeModelCatalogSyncState {
  status: CreativeModelCatalogSyncStatus;
  lastSyncedAt: string | null;
  lastError: string | null;
  lastStats: CreativeModelCatalogSyncStats | null;
}

/**
 * Subset of the wire contract (`AppModelCatalogItem`) the client mapper
 * consumes. Kept structural so the service never imports the generated SDK
 * types directly and stays decoupled from the transport layer.
 */
export interface RemoteCreativeModelItem {
  model: string;
  displayName: string;
  vendorCode: string;
  description: string | null;
  capabilityIntro?: string | null;
  modalities: string[];
  releaseStage: number | null;
  shelfState: number | null;
  routingState: number | null;
  replacementModel: string | null;
}

/** Wire enum mapping used by sdkwork-models (`model_catalog_import.rs`). */
export const CREATIVE_MODEL_RELEASE_STAGE = {
  ACTIVE: 1,
  PREVIEW: 2,
  DEPRECATED: 3,
  RETIRED: 4,
} as const;

export const CREATIVE_MODEL_SHELF_STATE = {
  LISTED: 1,
  HIDDEN: 2,
  ARCHIVED: 3,
} as const;
