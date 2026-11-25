#!/usr/bin/env node
/**
 * Render the SDKWork expert catalog into lifecycle-managed seed SQL.
 *
 * The expert catalog is authored in `sdkwork-specs`
 * (`specs/expert-catalog/expert-catalog.v1.json` and its persona bodies) and
 * described by `sdkwork-specs/EXPERTS_CATALOG_SPEC.md`. This generator is the
 * only writer of the expert seed assets:
 *
 * - `common/003_expert_catalog.sql` — expert categories, expert agents and their
 *   initial versions, expert teams and team members (locale-neutral base).
 * - `locales/zh-CN/002_expert_catalog.sql` — Chinese display, prompts and
 *   persona rows.
 * - `locales/en-US/002_expert_catalog.sql` — English display, prompts and
 *   persona rows.
 *
 * Every row id and uuid derives from a stable catalog key, so re-running the
 * generator is byte-identical for unchanged data and the seed checksums only
 * move when content moves. `--check` renders in memory and fails when the
 * committed seeds or the seed manifest drifted from the catalog.
 */

import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const HERE = path.dirname(fileURLToPath(import.meta.url))
const AGENTS_ROOT = path.resolve(HERE, '..')
const SPECS_ROOT = path.resolve(AGENTS_ROOT, '..', 'sdkwork-specs')
const CATALOG_PATH = path.join(SPECS_ROOT, 'specs', 'expert-catalog', 'expert-catalog.v1.json')
const PERSONAS_ROOT = path.join(SPECS_ROOT, 'specs', 'expert-catalog', 'personas')
const SEEDS_ROOT = path.join(AGENTS_ROOT, 'database', 'seeds')

const SEED_TIME = '2026-11-25T00:00:00+08:00'
const SEED_TIMESTAMP = `TIMESTAMPTZ '${SEED_TIME}'`
const TENANT_ID = 100001
const ORGANIZATION_ID = 0
const OWNER_USER_ID = 1
const CATALOG_REVISION = '1.0.0'
const DISPLAY_REVISION = '1.6.0'

/** Deterministic numeric id bases; each family owns a distinct range. */
const EXPERT_CATEGORY_ID_BASE = 541000000000000001n
const EXPERT_AGENT_ID_BASE = 366800000000000001n
const EXPERT_AGENT_VERSION_ID_BASE = 366900000000000001n
const EXPERT_TEAM_ID_BASE = 370000000000000001n
const EXPERT_TEAM_MEMBER_ID_BASE = 371000000000000001n
const EXPERT_PERSONA_ID_BASE = 372000000000000001n

/** Catalog locale id to persona directory. */
const PERSONA_LOCALES = [
  { locale: 'zh-CN', display: 'zh-CN' },
  { locale: 'en-US', display: 'en-US' },
]

const COMMON_SEED = 'common/003_expert_catalog.sql'
export const LOCALE_SEEDS = {
  'zh-CN': 'locales/zh-CN/002_expert_catalog.sql',
  'en-US': 'locales/en-US/002_expert_catalog.sql',
}

function fail(message) {
  throw new Error(`expert-seeds: ${message}`)
}

function assert(condition, message) {
  if (!condition) fail(message)
}

function sqlText(value) {
  return `'${String(value).replaceAll("'", "''")}'`
}

function sqlJson(value) {
  return `${sqlText(JSON.stringify(value))}::jsonb`
}

function sqlJsonArray(values) {
  return `${sqlText(JSON.stringify(values))}::jsonb`
}

function uuidFor(key) {
  const hex = createHash('sha256').update(key).digest('hex')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-4${hex.slice(13, 16)}-8${hex.slice(17, 20)}-${hex.slice(20, 32)}`
}

export function expertAgentId(slug) {
  return `agent.expert.${slug}`
}

function expertCode(slug) {
  return `expert-${slug}`
}

function categoryCode(division) {
  return `expert-${division}`
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex')
}

/** Read the authored catalog and validate the contract the seed relies on. */
export function readCatalog(catalogPath = CATALOG_PATH) {
  assert(existsSync(catalogPath), `missing catalog ${catalogPath}`)
  const catalog = JSON.parse(readFileSync(catalogPath, 'utf8'))
  assert(catalog.schemaVersion === '1.0.0', `unexpected schema ${catalog.schemaVersion}`)
  assert(Array.isArray(catalog.experts) && catalog.experts.length === 331, `expected 331 experts, got ${catalog.experts?.length}`)
  assert(Array.isArray(catalog.taxonomy?.divisions) && catalog.taxonomy.divisions.length === 22, 'expected 22 divisions')
  assert(Array.isArray(catalog.teams) && catalog.teams.length === 5, `expected 5 teams, got ${catalog.teams?.length}`)
  const slugs = new Set()
  for (const expert of catalog.experts) {
    assert(/^[a-z0-9][a-z0-9-]{1,127}$/u.test(expert.slug), `bad slug ${expert.slug}`)
    assert(!slugs.has(expert.slug), `duplicate slug ${expert.slug}`)
    slugs.add(expert.slug)
  }
  for (const team of catalog.teams) {
    assert(team.members.length >= 2 && team.members.length <= 8, `${team.id} member count`)
    for (const member of team.members) {
      assert(slugs.has(member.expertSlug), `${team.id} unknown expert ${member.expertSlug}`)
    }
  }
  return catalog
}

/**
 * Split a persona markdown file into its frontmatter and body.
 * @param text - the file content.
 * @returns the body without the leading frontmatter block.
 */
export function personaBody(text) {
  assert(text.startsWith('---'), 'persona is missing its frontmatter')
  const end = text.indexOf('\n---', 3)
  assert(end !== -1, 'persona frontmatter is unterminated')
  const after = text.indexOf('\n', end + 1)
  const body = (after === -1 ? '' : text.slice(after + 1)).trim()
  assert(body.length > 0, 'persona body is empty')
  return body
}

/** Read one persona body from the specs catalog. */
function readPersona(expert, locale) {
  const relative = expert.personas[locale]
  assert(typeof relative === 'string', `${expert.slug} has no ${locale} persona`)
  const file = path.join(PERSONAS_ROOT, locale, relative.split('/').slice(1).join('/'))
  assert(existsSync(file), `missing persona ${file}`)
  return personaBody(readFileSync(file, 'utf8'))
}

/**
 * Build the agent manifest for one expert.
 * @param expert - catalog expert.
 * @param catalog - the catalog (for its version).
 * @param locale - `zh` or `en` display selection.
 * @returns the manifest object.
 */
function agentManifest(expert, catalog, locale) {
  const zh = locale === 'zh'
  return {
    schema_version: '1.0.0',
    manifest_type: 'agent',
    agent_id: expertAgentId(expert.slug),
    name: expertCode(expert.slug),
    display_name: zh ? expert.name : expert.nameEn,
    description: zh ? expert.description : expert.descriptionEn,
    version: catalog.catalogVersion,
    domain: 'experts',
    agent_kind: 'conversation',
    category_code: categoryCode(expert.divisionCode),
    runtime: { family: 'sdkwork', engine: 'sdkwork', capabilities: ['model.chat'] },
    required_capabilities: ['model.chat'],
    event_families: ['agent.lifecycle'],
    expert: {
      slug: expert.slug,
      division_code: expert.divisionCode,
      emoji: expert.emoji,
      roster_version: catalog.catalogVersion,
      persona_locales: ['zh-CN', 'en-US'],
    },
    owner_name: 'SDKWork',
    status: 'active',
  }
}

/**
 * Render the locale-neutral expert catalog seed.
 * @param catalog - the catalog.
 * @returns the SQL text.
 */
export function renderCommonSeed(catalog) {
  const divisions = catalog.taxonomy.divisions
  const experts = catalog.experts
  const lines = []
  lines.push('-- SDKWork expert catalog: neutral category, agent, version, team and')
  lines.push('-- member base. Display names, descriptions, prompts and persona bodies')
  lines.push('-- live in the locale seeds.')
  lines.push('')
  lines.push(`-- Catalog revision: ${CATALOG_REVISION}.`)
  lines.push('')
  lines.push('-- Generated by scripts/generate-expert-seeds.mjs from')
  lines.push('-- ../sdkwork-specs/specs/expert-catalog/expert-catalog.v1.json.')
  lines.push('')
  lines.push('BEGIN;')
  lines.push('')

  for (const [index, division] of divisions.entries()) {
    const id = EXPERT_CATEGORY_ID_BASE + BigInt(index)
    lines.push('INSERT INTO ai_agent_category (')
    lines.push('  id, uuid, tenant_id, organization_id, category_code, name, description,')
    lines.push('  parent_id, sort_order, icon_ref, status, created_by, updated_by, version,')
    lines.push('  created_at, updated_at, deleted_at, deleted_by')
    lines.push(') VALUES (')
    lines.push(`  ${id}, ${sqlText(uuidFor(`expert.category.${division.code}`))}, ${TENANT_ID}, ${ORGANIZATION_ID},`)
    lines.push(`  ${sqlText(categoryCode(division.code))}, ${sqlText(division.nameEn)}, NULL,`)
    lines.push(`  NULL, ${(index + 1) * 10}, 'Users', 1, ${OWNER_USER_ID}, ${OWNER_USER_ID}, 0,`)
    lines.push(`  ${SEED_TIMESTAMP}, ${SEED_TIMESTAMP}, NULL, NULL`)
    lines.push(')')
    lines.push('ON CONFLICT (tenant_id, organization_id, category_code) DO UPDATE SET')
    lines.push('  sort_order = EXCLUDED.sort_order,')
    lines.push('  icon_ref = EXCLUDED.icon_ref,')
    lines.push('  status = EXCLUDED.status,')
    lines.push('  updated_at = EXCLUDED.updated_at,')
    lines.push('  deleted_at = NULL')
    lines.push(';')
    lines.push('')
  }

  for (const [index, expert] of experts.entries()) {
    const manifest = agentManifest(expert, catalog, 'en')
    lines.push('INSERT INTO ai_agent (')
    lines.push('  id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code,')
    lines.push('  display_name, description, manifest_json, manifest_schema_version,')
    lines.push('  default_code_task_intent_json, implementation_provider_id,')
    lines.push('  implementation_kind, implementation_type, status, visibility, tags_json,')
    lines.push('  allowed_role_codes, created_at, updated_at, deleted_at, version')
    lines.push(') VALUES (')
    lines.push(`  ${EXPERT_AGENT_ID_BASE + BigInt(index)}, ${sqlText(uuidFor(`expert.agent.${expert.slug}`))}, ${TENANT_ID}, ${ORGANIZATION_ID}, ${OWNER_USER_ID},`)
    lines.push(`  ${sqlText(expertAgentId(expert.slug))}, ${sqlText(expertCode(expert.slug))},`)
    lines.push(`  ${sqlText(expert.nameEn)}, ${sqlText(expert.descriptionEn)},`)
    lines.push(`  ${sqlJson(manifest)}, '1.0.0', NULL, NULL,`)
    lines.push(`  NULL, 'sdkwork-native', 1, 3, ${sqlJsonArray(['expert', 'conversation', expert.divisionCode, 'official', 'commercial'])},`)
    lines.push(`  '[]'::jsonb, ${SEED_TIMESTAMP}, ${SEED_TIMESTAMP}, NULL, 0`)
    lines.push(')')
    lines.push('ON CONFLICT (tenant_id, agent_id) DO UPDATE SET')
    lines.push('  code = EXCLUDED.code,')
    lines.push('  manifest_schema_version = EXCLUDED.manifest_schema_version,')
    lines.push('  implementation_kind = EXCLUDED.implementation_kind,')
    lines.push('  implementation_type = EXCLUDED.implementation_type,')
    lines.push('  status = EXCLUDED.status,')
    lines.push('  visibility = EXCLUDED.visibility,')
    lines.push('  tags_json = EXCLUDED.tags_json,')
    lines.push('  allowed_role_codes = EXCLUDED.allowed_role_codes,')
    lines.push('  updated_at = EXCLUDED.updated_at,')
    lines.push('  deleted_at = NULL')
    lines.push(';')
    lines.push('')

    lines.push('INSERT INTO ai_agent_version (')
    lines.push('  id, tenant_id, organization_id, agent_id, version_id, version_number,')
    lines.push('  manifest_json, default_code_task_intent_json, implementation_provider_id,')
    lines.push('  implementation_kind, implementation_type, description, created_by,')
    lines.push('  created_at, activated_at')
    lines.push(') VALUES (')
    lines.push(`  ${EXPERT_AGENT_VERSION_ID_BASE + BigInt(index)}, ${TENANT_ID}, ${ORGANIZATION_ID},`)
    lines.push(`  ${sqlText(expertAgentId(expert.slug))}, ${sqlText(`version.${expertCode(expert.slug)}.1`)}, 1,`)
    lines.push(`  ${sqlJson(manifest)}, NULL, NULL, NULL, 'sdkwork-native', NULL,`)
    lines.push(`  ${OWNER_USER_ID}, ${SEED_TIMESTAMP}, ${SEED_TIMESTAMP}`)
    lines.push(')')
    lines.push('ON CONFLICT (tenant_id, organization_id, agent_id, version_id) DO NOTHING')
    lines.push(';')
    lines.push('')
  }

  for (const [index, team] of catalog.teams.entries()) {
    lines.push('INSERT INTO ai_expert_team (')
    lines.push('  id, uuid, tenant_id, organization_id, team_id, name, name_en, description,')
    lines.push('  description_en, tags_json, tags_en_json, goal, goal_en, constraints_text,')
    lines.push('  delivery_requirements, delivery_requirements_en, examples_json,')
    lines.push('  examples_en_json, coordinator_mode, coordinator_template_id,')
    lines.push('  coordinator_template_version, status, sort_order, version, created_by,')
    lines.push('  updated_by, created_at, updated_at, deleted_at, deleted_by')
    lines.push(') VALUES (')
    lines.push(`  ${EXPERT_TEAM_ID_BASE + BigInt(index)}, ${sqlText(uuidFor(`expert.team.${team.id}`))}, ${TENANT_ID}, ${ORGANIZATION_ID},`)
    lines.push(`  ${sqlText(team.id)}, ${sqlText(team.name)}, ${sqlText(team.nameEn)},`)
    lines.push(`  ${sqlText(team.description)}, ${sqlText(team.descriptionEn)},`)
    lines.push(`  ${sqlJsonArray(team.tags)}, ${sqlJsonArray(team.tagsEn)},`)
    lines.push(`  ${sqlText(team.goal)}, ${sqlText(team.goalEn)}, ${sqlText(team.constraints)},`)
    lines.push(`  ${sqlText(team.deliveryRequirements)}, ${sqlText(team.deliveryRequirementsEn)},`)
    lines.push(`  ${sqlJsonArray(team.examples)}, ${sqlJsonArray(team.examplesEn)},`)
    lines.push(`  'template', ${sqlText(team.coordinator.templateId)}, ${team.coordinator.templateVersion},`)
    lines.push(`  1, ${(index + 1) * 10}, 0, ${OWNER_USER_ID}, ${OWNER_USER_ID},`)
    lines.push(`  ${SEED_TIMESTAMP}, ${SEED_TIMESTAMP}, NULL, NULL`)
    lines.push(')')
    lines.push('ON CONFLICT (tenant_id, organization_id, team_id) DO UPDATE SET')
    lines.push('  name = EXCLUDED.name,')
    lines.push('  name_en = EXCLUDED.name_en,')
    lines.push('  description = EXCLUDED.description,')
    lines.push('  description_en = EXCLUDED.description_en,')
    lines.push('  tags_json = EXCLUDED.tags_json,')
    lines.push('  tags_en_json = EXCLUDED.tags_en_json,')
    lines.push('  goal = EXCLUDED.goal,')
    lines.push('  goal_en = EXCLUDED.goal_en,')
    lines.push('  constraints_text = EXCLUDED.constraints_text,')
    lines.push('  delivery_requirements = EXCLUDED.delivery_requirements,')
    lines.push('  delivery_requirements_en = EXCLUDED.delivery_requirements_en,')
    lines.push('  examples_json = EXCLUDED.examples_json,')
    lines.push('  examples_en_json = EXCLUDED.examples_en_json,')
    lines.push('  coordinator_mode = EXCLUDED.coordinator_mode,')
    lines.push('  coordinator_template_id = EXCLUDED.coordinator_template_id,')
    lines.push('  coordinator_template_version = EXCLUDED.coordinator_template_version,')
    lines.push('  sort_order = EXCLUDED.sort_order,')
    lines.push('  updated_at = EXCLUDED.updated_at,')
    lines.push('  deleted_at = NULL')
    lines.push(';')
    lines.push('')

    for (const [position, member] of team.members.entries()) {
      const memberRow = EXPERT_TEAM_MEMBER_ID_BASE + BigInt(index * 10 + position)
      lines.push('INSERT INTO ai_expert_team_member (')
      lines.push('  id, uuid, tenant_id, organization_id, team_id, expert_agent_id,')
      lines.push('  position, duty, duty_en, instructions, instructions_en, created_at,')
      lines.push('  updated_at, deleted_at')
      lines.push(') VALUES (')
      lines.push(`  ${memberRow}, ${sqlText(uuidFor(`expert.team.${team.id}.member.${member.expertSlug}`))}, ${TENANT_ID}, ${ORGANIZATION_ID},`)
      lines.push(`  ${sqlText(team.id)}, ${sqlText(expertAgentId(member.expertSlug))}, ${position},`)
      lines.push(`  ${sqlText(member.duty)}, ${sqlText(member.dutyEn)},`)
      lines.push(`  ${sqlText(member.instructions)}, ${sqlText(member.instructionsEn)},`)
      lines.push(`  ${SEED_TIMESTAMP}, ${SEED_TIMESTAMP}, NULL`)
      lines.push(')')
      lines.push('ON CONFLICT (tenant_id, organization_id, team_id, expert_agent_id) DO UPDATE SET')
      lines.push('  position = EXCLUDED.position,')
      lines.push('  duty = EXCLUDED.duty,')
      lines.push('  duty_en = EXCLUDED.duty_en,')
      lines.push('  instructions = EXCLUDED.instructions,')
      lines.push('  instructions_en = EXCLUDED.instructions_en,')
      lines.push('  updated_at = EXCLUDED.updated_at,')
      lines.push('  deleted_at = NULL')
      lines.push(';')
      lines.push('')
    }
  }

  lines.push('COMMIT;')
  lines.push('')
  return lines.join('\n')
}

/**
 * Render one locale seed: category/agent display, agent versions, agent prompt
 * intent and the persona rows for that locale.
 * @param catalog - the catalog.
 * @param locale - `zh-CN` or `en-US`.
 * @returns the SQL text.
 */
export function renderLocaleSeed(catalog, locale) {
  const zh = locale === 'zh-CN'
  const lines = []
  lines.push(`-- SDKWork expert catalog: ${locale} display, prompts and persona bodies.`)
  lines.push('')
  lines.push(`-- Catalog revision: ${DISPLAY_REVISION}.`)
  lines.push('')
  lines.push('-- Generated by scripts/generate-expert-seeds.mjs from')
  lines.push('-- ../sdkwork-specs/specs/expert-catalog/expert-catalog.v1.json and its')
  lines.push('-- persona bodies. Do not hand-edit.')
  lines.push('')
  lines.push('BEGIN;')
  lines.push('')

  for (const division of catalog.taxonomy.divisions) {
    lines.push('UPDATE ai_agent_category SET')
    lines.push(`  name = ${sqlText(zh ? division.name : division.nameEn)},`)
    lines.push(`  description = NULL,`)
    lines.push(`  updated_at = ${SEED_TIMESTAMP}`)
    lines.push(`WHERE tenant_id = ${TENANT_ID} AND organization_id = ${ORGANIZATION_ID} AND category_code = ${sqlText(categoryCode(division.code))};`)
    lines.push('')
  }

  for (const [index, expert] of catalog.experts.entries()) {
    const manifest = agentManifest(expert, catalog, zh ? 'zh' : 'en')
    const body = readPersona(expert, locale)
    const personaRow = EXPERT_PERSONA_ID_BASE + BigInt(index * 2 + (zh ? 0 : 1))
    lines.push('UPDATE ai_agent SET')
    lines.push(`  display_name = ${sqlText(zh ? expert.name : expert.nameEn)},`)
    lines.push(`  description = ${sqlText(zh ? expert.description : expert.descriptionEn)},`)
    lines.push(`  manifest_json = ${sqlJson(manifest)},`)
    lines.push(`  updated_at = ${SEED_TIMESTAMP}`)
    lines.push(`WHERE tenant_id = ${TENANT_ID} AND agent_id = ${sqlText(expertAgentId(expert.slug))};`)
    lines.push('')
    lines.push('UPDATE ai_agent_version SET')
    lines.push(`  manifest_json = ${sqlJson(manifest)},`)
    lines.push('  description = NULL')
    lines.push(`WHERE tenant_id = ${TENANT_ID} AND organization_id = ${ORGANIZATION_ID}`)
    lines.push(`  AND agent_id = ${sqlText(expertAgentId(expert.slug))} AND version_id = ${sqlText(`version.${expertCode(expert.slug)}.1`)};`)
    lines.push('')
    lines.push('INSERT INTO ai_expert_persona (')
    lines.push('  id, uuid, tenant_id, organization_id, expert_agent_id, locale,')
    lines.push('  persona_text, content_sha256, created_at, updated_at')
    lines.push(') VALUES (')
    lines.push(`  ${personaRow}, ${sqlText(uuidFor(`expert.persona.${expert.slug}.${locale}`))}, ${TENANT_ID}, ${ORGANIZATION_ID},`)
    lines.push(`  ${sqlText(expertAgentId(expert.slug))}, ${sqlText(locale)},`)
    lines.push(`  ${sqlText(body)}, ${sqlText(sha256(body))},`)
    lines.push(`  ${SEED_TIMESTAMP}, ${SEED_TIMESTAMP}`)
    lines.push(')')
    lines.push('ON CONFLICT (tenant_id, organization_id, expert_agent_id, locale) DO UPDATE SET')
    lines.push('  persona_text = EXCLUDED.persona_text,')
    lines.push('  content_sha256 = EXCLUDED.content_sha256,')
    lines.push('  updated_at = EXCLUDED.updated_at')
    lines.push(';')
    lines.push('')
  }

  lines.push('COMMIT;')
  lines.push('')
  return lines.join('\n')
}

/**
 * Update the seed manifest with the expert seed files and their checksums.
 * @param manifestText - the current manifest text.
 * @param files - `{ relative, content }` for each seed file.
 * @returns the updated manifest text.
 */
export function renderSeedManifest(manifestText, files, readSeed = relative => readFileSync(path.join(SEEDS_ROOT, relative), 'utf8')) {
  const manifest = JSON.parse(manifestText)
  manifest.i18nVersion = DISPLAY_REVISION
  const byRelative = new Map(files.map(file => [file.relative, file.content]))
  const common = manifest.profiles.standard.common
  if (!common.includes('003_expert_catalog.sql')) common.push('003_expert_catalog.sql')
  for (const [locale, relative] of Object.entries(LOCALE_SEEDS)) {
    const basename = path.posix.basename(relative)
    const localeFiles = manifest.profiles.standard.locales[locale]
    if (!localeFiles.includes(basename)) localeFiles.push(basename)
    const set = manifest.localeSets[locale]
    set.version = DISPLAY_REVISION
    // `profile.locales` lists basenames resolved under `locales/<locale>/`;
    // `localeSets.files` lists seed-root-relative paths.
    if (!set.files.includes(relative)) set.files.push(relative)
    set.files.sort()
    const hash = createHash('sha256')
    for (const name of set.files) {
      const content = byRelative.get(name) ?? readSeed(name)
      hash.update(content)
    }
    set.checksum = `sha256:${hash.digest('hex')}`
  }
  return `${JSON.stringify(manifest, null, 2)}\n`
}

/** Render every seed file into memory. */
export function renderAll() {
  const catalog = readCatalog()
  return [
    { relative: COMMON_SEED, content: renderCommonSeed(catalog) },
    { relative: LOCALE_SEEDS['zh-CN'], content: renderLocaleSeed(catalog, 'zh-CN') },
    { relative: LOCALE_SEEDS['en-US'], content: renderLocaleSeed(catalog, 'en-US') },
  ]
}

/** Write every seed file and refresh the manifest. */
function writeAll(files) {
  for (const file of files) {
    const target = path.join(SEEDS_ROOT, file.relative)
    mkdirSync(path.dirname(target), { recursive: true })
    writeFileSync(target, file.content, 'utf8')
  }
  const manifestPath = path.join(SEEDS_ROOT, 'seed.manifest.json')
  writeFileSync(manifestPath, renderSeedManifest(readFileSync(manifestPath, 'utf8'), files), 'utf8')
}

/** Compare rendered files with what is on disk; returns drift messages. */
function checkAll(files) {
  const drifted = []
  for (const file of files) {
    const target = path.join(SEEDS_ROOT, file.relative)
    if (!existsSync(target) || readFileSync(target, 'utf8') !== file.content) {
      drifted.push(file.relative)
    }
  }
  const manifestPath = path.join(SEEDS_ROOT, 'seed.manifest.json')
  const expected = renderSeedManifest(readFileSync(manifestPath, 'utf8'), files)
  if (expected !== readFileSync(manifestPath, 'utf8')) drifted.push('seed.manifest.json')
  return drifted
}

if (process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const files = renderAll()
  if (process.argv.includes('--check')) {
    const drifted = checkAll(files)
    if (drifted.length > 0) {
      for (const file of drifted) console.error(`expert-seeds: ${file} is out of date; run scripts/generate-expert-seeds.mjs`)
      process.exit(1)
    }
    console.log('expert-seeds: seeds and manifest are up to date')
  } else {
    writeAll(files)
    const total = files.reduce((sum, file) => sum + file.content.length, 0)
    console.log(`expert-seeds: wrote ${files.length} files, ${total} bytes`)
  }
}
