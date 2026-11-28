#!/usr/bin/env node
/**
 * Static regression gate for the SDKWork expert catalog seeds.
 *
 * Verifies the authored catalog in `sdkwork-specs`, the generated seed assets,
 * and the seed manifest without a database connection. Live-database checks live
 * in `database/verification/expert_catalog_checks.sql`.
 *
 * Run `node scripts/generate-expert-seeds.mjs` to refresh the seeds; this gate
 * fails when they drifted from the catalog, when a row or stable key is missing,
 * or when the manifest no longer lists the files with their checksums.
 */

import { createHash } from 'node:crypto'
import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

import {
  LOCALE_SEEDS,
  expertAgentId,
  readCatalog,
  renderSeedManifest,
  renderAll,
} from './generate-expert-seeds.mjs'

const HERE = path.dirname(fileURLToPath(import.meta.url))
const AGENTS_ROOT = path.resolve(HERE, '..')
const SEEDS_ROOT = path.join(AGENTS_ROOT, 'database', 'seeds')

function fail(message) {
  throw new Error(`verify-expert-catalog: ${message}`)
}

function assert(condition, message) {
  if (!condition) fail(message)
}

function read(relative) {
  const absolute = path.join(SEEDS_ROOT, relative)
  assert(existsSync(absolute), `missing seed ${relative}`)
  return readFileSync(absolute, 'utf8')
}

function readManifest() {
  return JSON.parse(read('seed.manifest.json'))
}

function countMatches(text, pattern) {
  return (text.match(pattern) ?? []).length
}

/** Assert the generated seed files are byte-identical to a fresh render. */
function verifyDeterministicSeeds() {
  for (const file of renderAll()) {
    const committed = read(file.relative)
    assert(committed === file.content, `${file.relative} drifted from the catalog; run scripts/generate-expert-seeds.mjs`)
  }
}

/** Assert the manifest lists every expert seed and carries matching checksums. */
function verifyManifest(manifest, files) {
  const common = manifest.profiles.standard.common
  assert(common.includes('003_expert_catalog.sql'), 'standard profile does not seed the expert catalog')
  assert(common.includes('004_expert_market.sql'), 'standard profile does not seed the expert market')
  const byRelative = new Map(files.map(file => [file.relative, file.content]))
  for (const [locale, relative] of Object.entries(LOCALE_SEEDS)) {
    const basename = path.posix.basename(relative)
    assert(
      manifest.profiles.standard.locales[locale].includes(basename),
      `standard profile does not seed ${locale} expert content`,
    )
    const set = manifest.localeSets[locale]
    assert(set.files.includes(relative), `${locale} locale set does not list ${relative}`)
    const hash = createHash('sha256')
    for (const name of set.files) {
      const content = byRelative.get(name) ?? readFileSync(path.join(SEEDS_ROOT, name))
      hash.update(content)
    }
    assert(
      set.checksum === `sha256:${hash.digest('hex')}`,
      `${locale} locale checksum does not match its files`,
    )
    assert(set.version === manifest.i18nVersion, `${locale} locale version must equal i18nVersion`)
  }
  const expected = renderSeedManifest(read('seed.manifest.json'), files).trim()
  assert(JSON.stringify(manifest) === JSON.stringify(JSON.parse(expected)), 'seed manifest drifted from the catalog')
}

/** Assert the common seed carries the complete neutral catalog. */
function verifyCommonSeed(catalog) {
  const common = read('common/003_expert_catalog.sql')
  assert(countMatches(common, /INSERT INTO ai_agent_category \(/g) === catalog.taxonomy.divisions.length, 'category insert count')
  assert(countMatches(common, /INSERT INTO ai_agent \(/g) === catalog.experts.length, 'expert agent insert count')
  assert(countMatches(common, /INSERT INTO ai_agent_version \(/g) === catalog.experts.length, 'expert version insert count')
  assert(countMatches(common, /INSERT INTO ai_expert_team \(/g) === catalog.teams.length, 'expert team insert count')
  const members = catalog.teams.reduce((total, team) => total + team.members.length, 0)
  assert(countMatches(common, /INSERT INTO ai_expert_team_member \(/g) === members, 'expert team member insert count')
  for (const expert of catalog.experts) {
    assert(common.includes(`'${expertAgentId(expert.slug)}'`), `common seed is missing expert ${expert.slug}`)
  }
  for (const team of catalog.teams) {
    assert(common.includes(`'${team.id}'`), `common seed is missing team ${team.id}`)
  }
  for (const division of catalog.taxonomy.divisions) {
    assert(common.includes(`'expert-${division.code}'`), `common seed is missing division ${division.code}`)
  }
  assert(!/\bNaN\b/u.test(common) && !/\bundefined\b/u.test(common), 'common seed contains NaN/undefined')
}

/** Assert each locale seed localizes every expert and carries its persona row. */
function verifyLocaleSeed(catalog, locale) {
  const sql = read(LOCALE_SEEDS[locale])
  const basename = path.posix.basename(LOCALE_SEEDS[locale])
  assert(basename === '002_expert_catalog.sql', 'locale seed name')
  assert(countMatches(sql, /UPDATE ai_agent SET/g) === catalog.experts.length, `${locale} agent display update count`)
  assert(countMatches(sql, /UPDATE ai_agent_version SET/g) === catalog.experts.length, `${locale} version update count`)
  assert(countMatches(sql, /UPDATE ai_agent_category SET/g) === catalog.taxonomy.divisions.length, `${locale} category update count`)
  assert(countMatches(sql, /INSERT INTO ai_expert_persona \(/g) === catalog.experts.length, `${locale} persona insert count`)
  assert(countMatches(sql, /content_sha256, created_at, updated_at/g) === catalog.experts.length, `${locale} persona column header`)
  // Persona bodies are expert-authored markdown and may legitimately contain
  // code snippets with `undefined`, `NaN` or password examples, so the locale
  // seeds are gated by deterministic regeneration instead of source-shaped
  // content sentinels.
  for (const expert of catalog.experts) {
    assert(sql.includes(`'${expertAgentId(expert.slug)}'`), `${locale} seed is missing expert ${expert.slug}`)
  }
}

/** Assert every persona row carries a 64-character content hash. */
function verifyPersonaHashes(catalog) {
  for (const locale of Object.keys(LOCALE_SEEDS)) {
    const sql = read(LOCALE_SEEDS[locale])
    const hashes = [...sql.matchAll(/'([0-9a-f]{64})'/gu)].map(match => match[1])
    assert(hashes.length >= catalog.experts.length, `${locale} persona content hashes are missing`)
  }
}

/**
 * Assert the hand-maintained market seed carries one profile per expert and the
 * six curated collections with their three ordered members each.
 */
function verifyMarketSeed(catalog) {
  const market = read('common/004_expert_market.sql')
  const slugs = new Set(catalog.experts.map(expert => expert.slug))
  const knownIds = new Set(catalog.experts.map(expert => expertAgentId(expert.slug)))

  assert(countMatches(market, /INSERT INTO ai_expert_collection \(/g) === 6, 'expert collection insert count')
  assert(countMatches(market, /INSERT INTO ai_expert_collection_item \(/g) === 18, 'expert collection item insert count')
  assert(countMatches(market, /INSERT INTO ai_expert_profile \(/g) === catalog.experts.length, 'expert profile insert count')

  const collectionBlocks = market.split('INSERT INTO ai_expert_collection (').slice(1)
  assert(collectionBlocks.length === 6, 'expert collection block count')
  const collectionCodes = collectionBlocks.map(block => {
    const tokens = [...block.matchAll(/'(collection-[a-z][a-z0-9-]*)'/g)].map(match => match[1])
    const coverKeys = [...block.matchAll(/, '([a-z][a-z0-9-]*)',\r?\n/g)].map(match => match[1])
    assert(coverKeys.length === 1, 'collection row must carry one cover key')
    assert(
      coverKeys[0] === tokens[0].replace('collection-', ''),
      `${tokens[0]} cover key must equal the collection code suffix`,
    )
    return tokens[0]
  })
  for (const code of [
    'collection-ecommerce',
    'collection-content',
    'collection-investment',
    'collection-legal',
    'collection-smallbusiness',
    'collection-data',
  ]) {
    assert(collectionCodes.includes(code), `market seed is missing ${code}`)
  }

  // Every `agent.expert.*` reference must resolve to a real catalog slug.
  for (const match of market.matchAll(/'(agent\.expert\.[a-z0-9.-]+)'/g)) {
    assert(knownIds.has(match[1]), `market seed references unknown expert ${match[1]}`)
  }
  for (const block of market.split('INSERT INTO ai_expert_collection_item (').slice(1)) {
    const collectionCode = block.match(/'(collection-[a-z][a-z0-9-]*)'/)?.[1]
    const expertId = block.match(/'(agent\.expert\.[a-z0-9.-]+)'/)?.[1]
    assert(collectionCode, 'collection item is missing its collection code')
    assert(expertId && knownIds.has(expertId), `collection item references unknown expert ${expertId}`)
  }
  for (const block of market.split('INSERT INTO ai_expert_profile (').slice(1)) {
    const expertId = block.match(/'(agent\.expert\.[a-z0-9.-]+)'/)?.[1]
    assert(expertId && slugs.has(expertId.replace('agent.expert.', '')), `profile references unknown expert ${expertId}`)
  }
  assert(!/\bNaN\b/u.test(market) && !/\bundefined\b/u.test(market), 'market seed contains NaN/undefined')
}

verifyDeterministicSeeds()
const catalog = readCatalog()
const manifest = readManifest()
const files = renderAll()
verifyManifest(manifest, files)
verifyCommonSeed(catalog)
for (const locale of Object.keys(LOCALE_SEEDS)) verifyLocaleSeed(catalog, locale)
verifyPersonaHashes(catalog)
verifyMarketSeed(catalog)
process.stdout.write(
  `verify-expert-catalog: ${catalog.experts.length} experts, ${catalog.taxonomy.divisions.length} divisions, `
  + `${catalog.teams.length} teams, ${catalog.teams.reduce((total, team) => total + team.members.length, 0)} members, `
  + `6 collections, ${catalog.experts.length} market profiles, seeds and manifest verified\n`,
)
