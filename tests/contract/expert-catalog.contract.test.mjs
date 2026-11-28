/**
 * Contract tests for the expert catalog seed pipeline: the authored catalog
 * shape, deterministic rendering, persona body extraction and manifest
 * checksums. The static gate (`scripts/verify-expert-catalog.mjs`) covers the
 * committed files; these tests cover the generator itself.
 */
import assert from 'node:assert/strict'
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { describe, it } from 'node:test'
import { fileURLToPath } from 'node:url'
import {
  LOCALE_SEEDS,
  expertAgentId,
  personaBody,
  readCatalog,
  renderAll,
  renderSeedManifest,
} from '../../scripts/generate-expert-seeds.mjs'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const marketSeed = readFileSync(
  path.join(repoRoot, 'database/seeds/common/004_expert_market.sql'),
  'utf8',
)

describe('expert catalog seeds', () => {
  const catalog = readCatalog()

  it('reads the 331-expert, 22-division, 5-team catalog', () => {
    assert.equal(catalog.experts.length, 331)
    assert.equal(catalog.taxonomy.divisions.length, 22)
    assert.equal(catalog.teams.length, 5)
    assert.equal(new Set(catalog.experts.map(expert => expert.slug)).size, 331)
  })

  it('renders deterministically', () => {
    assert.deepEqual(renderAll(), renderAll())
  })

  it('renders one neutral seed and two locale seeds', () => {
    const files = renderAll()
    assert.deepEqual(files.map(file => file.relative), [
      'common/003_expert_catalog.sql',
      LOCALE_SEEDS['zh-CN'],
      LOCALE_SEEDS['en-US'],
    ])
    for (const file of files) {
      assert.ok(file.content.startsWith('-- SDKWork expert catalog'))
      assert.ok(file.content.includes('COMMIT;'))
    }
  })

  it('strips the persona frontmatter from the seeded body', () => {
    assert.equal(personaBody('---\nname: X\n---\n\n# Body\n\ntext\n'), '# Body\n\ntext')
    assert.throws(() => personaBody('no frontmatter'))
  })

  it('adds both locale seeds to the manifest with matching checksums', () => {
    const files = renderAll()
    const base = JSON.stringify({
      schemaVersion: 1,
      i18nVersion: '1.5.0',
      localeSets: {
        'zh-CN': { version: '1.5.0', checksum: 'sha256:x', files: ['locales/zh-CN/001_agent_catalog_display.sql'] },
        'en-US': { version: '1.5.0', checksum: 'sha256:y', files: ['locales/en-US/001_agent_catalog_display.sql'] },
      },
      profiles: { standard: { common: ['001_bootstrap.sql'], locales: { 'zh-CN': [], 'en-US': [] } } },
    })
    const once = renderSeedManifest(base, files, () => '')
    const twice = renderSeedManifest(once, files, () => '')
    assert.equal(once, twice)
    const manifest = JSON.parse(once)
    assert.equal(manifest.i18nVersion, '1.6.0')
    assert.ok(manifest.profiles.standard.common.includes('003_expert_catalog.sql'))
    for (const locale of Object.keys(LOCALE_SEEDS)) {
      assert.ok(manifest.profiles.standard.locales[locale].includes('002_expert_catalog.sql'))
      assert.ok(manifest.localeSets[locale].files.includes(LOCALE_SEEDS[locale]))
      assert.match(manifest.localeSets[locale].checksum, /^sha256:[0-9a-f]{64}$/u)
    }
  })

  it('rejects a catalog whose expert count is wrong', () => {
    const directory = mkdtempSync(path.join(tmpdir(), 'expert-catalog-'))
    try {
      const file = path.join(directory, 'catalog.json')
      writeFileSync(file, JSON.stringify({ schemaVersion: '1.0.0', taxonomy: { divisions: [] }, experts: [], teams: [] }))
      assert.throws(() => readCatalog(file), /expected 331 experts/u)
    } finally {
      rmSync(directory, { recursive: true, force: true })
    }
  })

  it('seeds one market profile per expert with division-local sort order', () => {
    const divisionPosition = new Map()
    for (const expert of catalog.experts) {
      const expected = divisionPosition.get(expert.divisionCode) ?? 0
      divisionPosition.set(expert.divisionCode, expected + 1)
      const match = marketSeed.match(
        new RegExp(
          `'${expertAgentId(expert.slug)}',\\s*(\\d+),\\s*(?:NULL|'[a-z][a-z0-9-]*'),\\s*(\\d+),\\s*TIMESTAMPTZ`,
          'u',
        ),
      )
      assert.ok(match, `missing market profile for ${expert.slug}`)
      assert.ok(Number(match[1]) > 0, `usage count for ${expert.slug} must be positive`)
      assert.equal(Number(match[2]), expected, `sort order for ${expert.slug}`)
    }
    assert.equal(
      (marketSeed.match(/INSERT INTO ai_expert_profile \(/gu) ?? []).length,
      catalog.experts.length,
    )
  })

  it('seeds six curated collections of three real catalog experts', () => {
    const codes = [
      'collection-ecommerce',
      'collection-content',
      'collection-investment',
      'collection-legal',
      'collection-smallbusiness',
      'collection-data',
    ]
    const slugs = new Set(catalog.experts.map(expert => expert.slug))

    const collectionBlocks = marketSeed
      .split('INSERT INTO ai_expert_collection (')
      .slice(1)
    assert.equal(collectionBlocks.length, 6)
    for (const block of collectionBlocks) {
      const collectionCode = block.match(/'(collection-[a-z][a-z0-9-]*)'/u)?.[1]
      const coverKey = block.match(/, '([a-z][a-z0-9-]*)',\r?\n/u)?.[1]
      assert.ok(codes.includes(collectionCode), `unexpected collection ${collectionCode}`)
      assert.equal(
        coverKey,
        collectionCode.replace('collection-', ''),
        `${collectionCode} coverKey must equal the collection code suffix`,
      )
    }

    const itemBlocks = marketSeed
      .split('INSERT INTO ai_expert_collection_item (')
      .slice(1)
    assert.equal(itemBlocks.length, 18)
    for (const block of itemBlocks) {
      const collectionCode = block.match(/'(collection-[a-z][a-z0-9-]*)'/u)?.[1]
      const agentId = block.match(/'(agent\.expert\.[a-z0-9.-]+)'/u)?.[1]
      assert.ok(codes.includes(collectionCode), `unknown collection ${collectionCode}`)
      assert.ok(
        agentId && slugs.has(agentId.replace('agent.expert.', '')),
        `unknown collection member ${agentId}`,
      )
    }
  })
})
