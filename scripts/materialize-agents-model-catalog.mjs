#!/usr/bin/env node
/**
 * Materialize the sdkwork-models static catalog into an agents-service
 * snapshot consumed by `GET /app/v3/api/ai/models` (operationId `models.list`).
 *
 * Source of truth: `../sdkwork-models/models` (catalogVersion tracked in the
 * snapshot). The snapshot is a committed build input
 * (CODE_STYLE_SPEC.md §7 build source integrity): regenerate it explicitly
 * with `node scripts/materialize-agents-model-catalog.mjs` after the
 * sdkwork-models catalog changes.
 *
 * The projection mirrors the models app-api wire contract
 * (`AppModelCatalogItem`): string lifecycle enums are encoded to the same
 * integer codes as `sdkwork-models-catalog-repository-sqlx::model_catalog_import`,
 * and `modalities` is the sorted union of input/output modalities plus the
 * primary capability (`model_modality_codes`).
 * @module scripts/materialize-agents-model-catalog
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const modelsRoot = resolve(root, '../sdkwork-models/models')
const outputPath = resolve(
  root,
  'crates/sdkwork-intelligence-agents-service/specs/model-catalog.snapshot.json',
)

const RELEASE_STAGE_CODES = { active: 1, preview: 2, deprecated: 3, retired: 4 }
const SHELF_STATE_CODES = { listed: 1, hidden: 2, archived: 3 }
const ROUTING_STATE_CODES = { enabled: 1 }

function requireReleaseStage(value) {
  const code = RELEASE_STAGE_CODES[value]
  if (code === undefined) {
    throw new Error(`Unknown releaseStage ${JSON.stringify(value)}`)
  }
  return code
}

function requireShelfState(value) {
  const code = SHELF_STATE_CODES[value]
  if (code === undefined) {
    throw new Error(`Unknown shelfState ${JSON.stringify(value)}`)
  }
  return code
}

function modelModalityCodes(model) {
  const codes = new Set()
  for (const modality of [...model.inputModalities, ...model.outputModalities]) {
    const value = typeof modality === 'string' ? modality.trim() : ''
    if (value) codes.add(value)
  }
  const primary = typeof model.primaryCapability === 'string' ? model.primaryCapability.trim() : ''
  if (primary) codes.add(primary)
  return [...codes].sort()
}

function projectModel(model) {
  return {
    model: model.modelId,
    catalogKey: model.catalogKey,
    displayName: model.displayName,
    vendor: model.vendorName ?? model.vendorCode,
    vendorCode: model.vendorCode,
    description: model.description ?? null,
    capabilityIntro: null,
    capabilities: [...(model.capabilities ?? [])].sort(),
    modalities: modelModalityCodes(model),
    inputModalities: [...(model.inputModalities ?? [])],
    outputModalities: [...(model.outputModalities ?? [])],
    apiFormat: model.apiFormat ?? '',
    releaseStage: requireReleaseStage(model.releaseStage),
    shelfState: requireShelfState(model.shelfState),
    routingState: ROUTING_STATE_CODES[model.routingState] ?? 0,
    replacementModel: model.replacementModel ?? null,
    supportsStreaming: model.supportsStreaming ?? false,
    supportsTools: model.supportsTools ?? false,
    supportsJsonSchema: model.supportsJsonSchema ?? false,
  }
}

const indexPath = resolve(modelsRoot, 'index.json')
if (!existsSync(indexPath)) {
  throw new Error(`sdkwork-models index not found at ${indexPath}`)
}
const index = JSON.parse(readFileSync(indexPath, 'utf-8'))

const items = []
for (const vendor of index.vendors ?? []) {
  for (const relativePath of vendor.modelFiles ?? []) {
    const modelPath = resolve(modelsRoot, relativePath)
    if (!existsSync(modelPath)) {
      throw new Error(`Model file listed in index.json is missing: ${relativePath}`)
    }
    const model = JSON.parse(readFileSync(modelPath, 'utf-8'))
    items.push(projectModel(model))
  }
}

if (typeof index.modelCount === 'number' && items.length !== index.modelCount) {
  throw new Error(
    `Catalog drift: index.json modelCount=${index.modelCount} but materialized ${items.length} models`,
  )
}

items.sort((left, right) => left.catalogKey.localeCompare(right.catalogKey))

const snapshot = {
  schemaVersion: 1,
  source: 'sdkwork-models',
  catalogVersion: index.catalogVersion,
  generatedAt: new Date().toISOString(),
  modelCount: items.length,
  items,
}

mkdirSync(dirname(outputPath), { recursive: true })
writeFileSync(outputPath, `${JSON.stringify(snapshot, null, 2)}\n`, 'utf-8')
console.log(`materialized ${items.length} models (catalog ${index.catalogVersion}) -> ${outputPath}`)
