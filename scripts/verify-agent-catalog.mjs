#!/usr/bin/env node
/**
 * Static regression gate for the SDKWork official agent catalog.
 *
 * Verifies both the source catalog and the rendered seed assets, without a
 * database connection. The live-database checks live in
 * `database/verification/agent_catalog_checks.sql`.
 */

import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  AGENTS,
  CATALOG_VERSION,
  MCP_CATEGORIES,
  MCP_SERVERS,
} from "./agent-catalog/catalog.mjs";
import {
  agentId,
  buildManagementProfile,
  buildPrompt,
  knowledgeKey,
  knowledgeSpaceId,
  memorySpaceId,
  skillId,
  validateCatalog,
  WORKSPACE_ROOT,
} from "./generate-agent-catalog-seeds.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const AGENTS_ROOT = path.resolve(HERE, "..");

const REQUIRED_COVERAGE = {
  engineering: 5,
  education: 14,
  healthcare: 3,
  legal: 4,
  travel: 2,
  ecommerce: 3,
  realestate: 2,
  restaurant: 3,
  enterprise: 3,
  government: 3,
  hospitality: 3,
};

function fail(message) {
  throw new Error(message);
}

function assert(condition, message) {
  if (!condition) fail(message);
}

function read(relative) {
  const absolute = path.join(AGENTS_ROOT, relative);
  assert(existsSync(absolute), `missing file: ${relative}`);
  return readFileSync(absolute, "utf8");
}

function readJson(relative) {
  return JSON.parse(read(relative));
}

function sha256File(absolute) {
  return createHash("sha256").update(readFileSync(absolute)).digest("hex");
}

function expectedLocaleChecksum(seedsRoot, files) {
  if (files.length === 0) {
    return `sha256:${createHash("sha256").update("[]").digest("hex")}`;
  }
  const hash = createHash("sha256");
  for (const relative of files) {
    hash.update(readFileSync(path.join(seedsRoot, relative)));
  }
  return `sha256:${hash.digest("hex")}`;
}

function countMatches(text, pattern) {
  return (text.match(pattern) ?? []).length;
}

function verifyCatalog() {
  validateCatalog();
  assert(AGENTS.length === 58, `expected 58 agents, got ${AGENTS.length}`);
  assert(MCP_SERVERS.length === 16, `expected 16 MCP servers, got ${MCP_SERVERS.length}`);
  const categoryCounts = new Map();
  for (const agent of AGENTS) {
    categoryCounts.set(agent.category, (categoryCounts.get(agent.category) ?? 0) + 1);
    assert(agent.tags.length >= 2, `${agent.slug} needs at least two taxonomy tags`);
    assert(agent.mcp.length >= 1, `${agent.slug} lacks MCP binding`);
    assert(agent.mcp.length <= 3, `${agent.slug} has too many MCP bindings`);
    assert(agent.temperature >= 0 && agent.temperature <= 2, `${agent.slug} temperature out of range`);
    assert(agent.icon && agent.color, `${agent.slug} icon/color required`);
    assert(/^#[0-9A-Fa-f]{6}$/.test(agent.color), `${agent.slug} color must be hex`);
    assert(agent.zh.suggestions.length <= 12 && agent.en.suggestions.length <= 12, `${agent.slug} suggestion limit`);
    assert(agent.zh.suggestions.length >= 3 && agent.en.suggestions.length >= 3, `${agent.slug} needs 3+ suggestions`);
    for (const locale of ["zh", "en"]) {
      const prompt = buildPrompt(agent, locale);
      assert(prompt.length >= 900, `${agent.slug}.${locale} prompt too short`);
      assert(prompt.includes("## "), `${agent.slug}.${locale} prompt lacks sections`);
      assert(!prompt.includes("undefined"), `${agent.slug}.${locale} prompt contains undefined`);
      const profile = buildManagementProfile(agent, locale);
      assert(profile.knowledgeBaseIds.length === 1, `${agent.slug}.${locale} needs one KB`);
      assert(profile.knowledgeBaseIds[0].startsWith("knowledge.base."), `${agent.slug}.${locale} KB id prefix`);
      assert(profile.skillIds.length === 1, `${agent.slug}.${locale} needs one skill`);
      assert(profile.skillIds[0].startsWith("skill."), `${agent.slug}.${locale} skill id prefix`);
      assert(profile.model.startsWith("model."), `${agent.slug}.${locale} model id prefix`);
      assert(profile.type === "normal", `${agent.slug}.${locale} agent type`);
      assert(profile.knowledgeBaseIds[0] === knowledgeKey(agent), `${agent.slug}.${locale} KB key mismatch`);
      assert(profile.skillIds[0] === skillId(agent), `${agent.slug}.${locale} skill mismatch`);
    }
    assert(
      !/[\u4e00-\u9fff]/.test(JSON.stringify(agent.en)),
      `${agent.slug} English payload contains Chinese characters`,
    );
    assert(
      buildPrompt(agent, "zh").includes("不可信参考数据"),
      `${agent.slug} Chinese prompt lacks the untrusted-evidence guardrail`,
    );
    assert(
      buildPrompt(agent, "en").includes("untrusted reference data"),
      `${agent.slug} English prompt lacks the untrusted-evidence guardrail`,
    );
  }
  for (const [category, minimum] of Object.entries(REQUIRED_COVERAGE)) {
    const actual = categoryCounts.get(category) ?? 0;
    assert(actual >= minimum, `category ${category} has ${actual}, expected >= ${minimum}`);
  }
  const k12 = AGENTS.filter((agent) => agent.slug.startsWith("edu-k12-")).length;
  const university = AGENTS.filter((agent) => agent.slug.startsWith("edu-uni-")).length;
  assert(k12 >= 8, `K12 coverage too small: ${k12}`);
  assert(university >= 6, `university coverage too small: ${university}`);

  const ids = AGENTS.map(agentId);
  assert(new Set(ids).size === ids.length, "duplicate agent ids");
  const codes = AGENTS.map((agent) => agent.slug);
  assert(new Set(codes).size === codes.length, "duplicate agent codes");
  const prompts = new Set();
  for (const agent of AGENTS) {
    for (const locale of ["zh", "en"]) {
      const prompt = buildPrompt(agent, locale);
      assert(!prompts.has(prompt), `duplicate ${locale} prompt for ${agent.slug}`);
      prompts.add(prompt);
    }
  }
}

function verifyAgentsSql() {
  const common = read("database/seeds/common/002_agent_catalog.sql");
  const zh = read("database/seeds/locales/zh-CN/001_agent_catalog_display.sql");
  const en = read("database/seeds/locales/en-US/001_agent_catalog_display.sql");
  assert(countMatches(common, /INSERT INTO ai_agent \(/g) === AGENTS.length, "agent insert count");
  assert(countMatches(common, /INSERT INTO ai_agent_version \(/g) === AGENTS.length, "version insert count");
  const expectedSlots = AGENTS.reduce((total, agent) => total + 3 + agent.mcp.length, 0);
  assert(
    countMatches(common, /INSERT INTO ai_agent_composition_slot \(/g) === expectedSlots,
    "composition slot insert count",
  );
  assert(countMatches(common, /ON CONFLICT \(tenant_id, agent_id\) DO UPDATE SET/g) === AGENTS.length, "agent upsert count");
  for (const [locale, sql] of [["zh", zh], ["en", en]]) {
    assert(countMatches(sql, /UPDATE ai_agent SET/g) === AGENTS.length, `${locale} agent update count`);
    assert(countMatches(sql, /UPDATE ai_agent_version SET/g) === AGENTS.length, `${locale} version update count`);
    assert(countMatches(sql, /UPDATE ai_agent_composition_slot SET/g) === AGENTS.length, `${locale} skill update count`);
    assert(!sql.includes("NaN") && !sql.includes("undefined"), `${locale} SQL contains NaN/undefined`);
  }
  assert(!common.includes("NaN") && !common.includes("undefined"), "generated SQL contains NaN/undefined");
  assert(!common.includes("systemPrompt"), "locale-specific system prompts must not appear in the common seed");
  assert(!common.includes("Knowledge Base"), "locale-specific knowledge display must not appear in the common seed");
  assert(!/\bpassword\s*=/i.test(common), "generated SQL contains a password assignment");
  assert(!/AKIA[0-9A-Z]{16}/.test(common), "generated SQL contains an AWS key");
  assert(!/sk-[A-Za-z0-9]{16,}/.test(common), "generated SQL contains an API key");
  for (const [index, agent] of AGENTS.entries()) {
    const id = agentId(agent);
    assert(common.includes(`'${id}'`), `SQL missing agent ${id}`);
    assert(common.includes(`'${knowledgeSpaceId(index)}'`), `SQL missing KB ref for ${agent.slug}`);
    assert(common.includes(`'space:${memorySpaceId(index)}'`), `SQL missing memory ref for ${agent.slug}`);
    for (const key of agent.mcp) {
      assert(common.includes(`'${key}'`), `SQL missing MCP ref ${key} for ${agent.slug}`);
    }
  }
  const policySpaceCount = countMatches(common, /"spaceId":"\d{18}"/g);
  assert(policySpaceCount === AGENTS.length, `knowledge policy spaceId count ${policySpaceCount}`);
  assert(!/"spaceId":\d{18}/.test(common), "spaceId must be a JSON string to avoid int64 precision loss");
  const mcpSlotCount = AGENTS.reduce((total, agent) => total + agent.mcp.length, 0);
  assert(countMatches(common, /"connectorRef":"connector\./g) === mcpSlotCount, "MCP connector references missing");
  assert(countMatches(common, /"activationState":"catalog_draft"/g) === mcpSlotCount, "MCP activation state missing");
}

function verifySeedManifest(repo, config) {
  const seedsRoot = path.join(WORKSPACE_ROOT, repo, "database", "seeds");
  const manifest = JSON.parse(readFileSync(path.join(seedsRoot, "seed.manifest.json"), "utf8"));
  assert(manifest.kind === "sdkwork.database.seed", `${repo} manifest kind`);
  assert(manifest.defaultLocale === "zh-CN", `${repo} default locale`);
  assert(manifest.fallbackLocale === "zh-CN", `${repo} fallback locale`);
  assert(Array.isArray(manifest.activeLocales) && manifest.activeLocales.includes("zh-CN"), `${repo} active locale`);
  assert(manifest.profiles?.standard, `${repo} standard profile missing`);
  for (const file of config.commonFiles) {
    assert(manifest.profiles.standard.common.includes(file), `${repo} common seed missing ${file}`);
  }
  assert(
    manifest.profiles.standard.locales["zh-CN"].includes(config.zhFile),
    `${repo} zh locale seed missing`,
  );
  assert(
    manifest.profiles.standard.locales["en-US"].includes(config.enFile),
    `${repo} en locale seed missing`,
  );
  for (const [locale, file] of [["zh-CN", config.zhFile], ["en-US", config.enFile]]) {
    const localeSet = manifest.localeSets[locale];
    assert(localeSet, `${repo} localeSets ${locale} missing`);
    assert(localeSet.required === true, `${repo} ${locale} localeSet must be required`);
    assert(localeSet.files.includes(`locales/${locale}/${file}`), `${repo} ${locale} localeSet file missing`);
    const expected = expectedLocaleChecksum(seedsRoot, localeSet.files);
    assert(localeSet.checksum === expected, `${repo} ${locale} locale checksum mismatch`);
  }
  return manifest;
}

function verifyKnowledgebaseSeed() {
  const sql = read("../sdkwork-knowledgebase/database/seeds/common/002_agent_catalog.sql");
  const zh = read("../sdkwork-knowledgebase/database/seeds/locales/zh-CN/001_agent_catalog_display.sql");
  const en = read("../sdkwork-knowledgebase/database/seeds/locales/en-US/001_agent_catalog_display.sql");
  assert(countMatches(sql, /INSERT INTO kb_space \(/g) === AGENTS.length, "kb_space insert count");
  assert(countMatches(zh, /UPDATE kb_space SET/g) === AGENTS.length, "kb_space zh update count");
  assert(countMatches(en, /UPDATE kb_space SET/g) === AGENTS.length, "kb_space en update count");
  assert(sql.includes("set_config('app.current_tenant_id'"), "KB seed must set tenant RLS context");
  for (const [index, agent] of AGENTS.entries()) {
    assert(sql.includes(String(knowledgeSpaceId(index))), `KB seed missing ${agent.slug}`);
    assert(zh.includes(String(knowledgeSpaceId(index))), `KB zh seed missing ${agent.slug}`);
  }
}

function verifyMemorySeed() {
  const sql = read("../sdkwork-memory/database/seeds/common/002_agent_catalog.sql");
  const zh = read("../sdkwork-memory/database/seeds/locales/zh-CN/001_agent_catalog_display.sql");
  const en = read("../sdkwork-memory/database/seeds/locales/en-US/001_agent_catalog_display.sql");
  assert(countMatches(sql, /INSERT INTO ai_space \(/g) === AGENTS.length, "ai_space insert count");
  assert(countMatches(sql, /INSERT INTO ai_subject \(/g) === AGENTS.length, "ai_subject insert count");
  assert(countMatches(sql, /INSERT INTO ai_memory_binding \(/g) === AGENTS.length, "ai_memory_binding insert count");
  assert(sql.includes("'agent_space'"), "memory bindings must use the agent_space binding kind");
  assert(sql.includes("'retriever'"), "memory bindings must use the retriever role");
  assert(!sql.includes("'agent_scope'"), "agent_scope is not a valid memory binding kind");
  assert(countMatches(zh, /UPDATE ai_space SET/g) === AGENTS.length, "memory zh space update count");
  assert(countMatches(en, /UPDATE ai_space SET/g) === AGENTS.length, "memory en space update count");
  for (const [index, agent] of AGENTS.entries()) {
    assert(sql.includes(String(memorySpaceId(index))), `memory seed missing ${agent.slug}`);
  }
}

function verifyMcpSeed() {
  const sql = read("../sdkwork-mcp/database/seeds/common/002_agent_catalog.sql");
  const zh = read("../sdkwork-mcp/database/seeds/locales/zh-CN/001_agent_catalog_display.sql");
  const en = read("../sdkwork-mcp/database/seeds/locales/en-US/001_agent_catalog_display.sql");
  assert(countMatches(sql, /INSERT INTO ai_mcp_server \(/g) === MCP_SERVERS.length, "MCP server insert count");
  assert(countMatches(sql, /INSERT INTO ai_mcp_server_category \(/g) === MCP_CATEGORIES.length, "MCP category insert count");
  assert(countMatches(sql, /INSERT INTO ai_mcp_connector \(/g) === MCP_SERVERS.length, "MCP connector insert count");
  assert(countMatches(zh, /UPDATE ai_mcp_server SET/g) === MCP_SERVERS.length, "MCP zh update count");
  assert(countMatches(en, /UPDATE ai_mcp_server SET/g) === MCP_SERVERS.length, "MCP en update count");
  assert(countMatches(zh, /UPDATE ai_mcp_server_category SET/g) === MCP_CATEGORIES.length, "MCP zh category update count");
  assert(countMatches(en, /UPDATE ai_mcp_server_category SET/g) === MCP_CATEGORIES.length, "MCP en category update count");
  for (const server of MCP_SERVERS) {
    assert(sql.includes(`'${server.key}'`), `MCP seed missing ${server.key}`);
    assert(zh.includes(`'${server.key}'`), `MCP zh seed missing ${server.key}`);
    assert(sql.includes(`'connector.${server.key.replace("mcp.sdkwork.", "")}'`), `MCP connector missing ${server.key}`);
  }
  for (const category of MCP_CATEGORIES) {
    assert(sql.includes(`'${category.code}'`), `MCP category missing ${category.code}`);
  }
}

function verifyCrossModuleReferences() {
  const kbSql = read("../sdkwork-knowledgebase/database/seeds/common/002_agent_catalog.sql");
  const memorySql = read("../sdkwork-memory/database/seeds/common/002_agent_catalog.sql");
  const mcpSql = read("../sdkwork-mcp/database/seeds/common/002_agent_catalog.sql");
  for (const [index, agent] of AGENTS.entries()) {
    assert(kbSql.includes(String(knowledgeSpaceId(index))), `KB resource missing for ${agent.slug}`);
    assert(memorySql.includes(String(memorySpaceId(index))), `memory resource missing for ${agent.slug}`);
  }
  for (const server of MCP_SERVERS) {
    assert(mcpSql.includes(`'${server.key}'`), `MCP resource missing for ${server.key}`);
  }
}

function verifyDatabaseManifest() {
  const dbManifest = readJson("database/database.manifest.json");
  const seedManifest = readJson("database/seeds/seed.manifest.json");
  assert(
    JSON.stringify(dbManifest.lifecycle.activeSeedLocales) === JSON.stringify(seedManifest.activeLocales),
    "database.manifest activeSeedLocales must match seed.manifest activeLocales",
  );
  assert(dbManifest.lifecycle.defaultSeedLocale === seedManifest.defaultLocale, "default seed locale mismatch");
  assert(dbManifest.lifecycle.defaultSeedProfile === "standard", "default seed profile must be standard");
}

export function verifyAgentCatalog() {
  verifyCatalog();
  verifyAgentsSql();
  verifySeedManifest("sdkwork-agents", {
    commonFiles: ["001_bootstrap.sql", "002_agent_catalog.sql"],
    zhFile: "001_agent_catalog_display.sql",
    enFile: "001_agent_catalog_display.sql",
  });
  verifySeedManifest("sdkwork-knowledgebase", {
    commonFiles: ["001_bootstrap.sql", "002_agent_catalog.sql"],
    zhFile: "001_agent_catalog_display.sql",
    enFile: "001_agent_catalog_display.sql",
  });
  verifySeedManifest("sdkwork-memory", {
    commonFiles: ["common/001_bootstrap.sql", "common/002_agent_catalog.sql"],
    zhFile: "001_agent_catalog_display.sql",
    enFile: "001_agent_catalog_display.sql",
  });
  verifySeedManifest("sdkwork-mcp", {
    commonFiles: ["001_bootstrap.sql", "002_agent_catalog.sql"],
    zhFile: "001_agent_catalog_display.sql",
    enFile: "001_agent_catalog_display.sql",
  });
  verifyKnowledgebaseSeed();
  verifyMemorySeed();
  verifyMcpSeed();
  verifyCrossModuleReferences();
  verifyDatabaseManifest();
  return {
    catalogVersion: CATALOG_VERSION,
    agents: AGENTS.length,
    mcpServers: MCP_SERVERS.length,
    categories: [...new Set(AGENTS.map((agent) => agent.category))].sort(),
  };
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  const result = verifyAgentCatalog();
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}
