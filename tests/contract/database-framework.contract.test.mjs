import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { validateDatabaseFramework } from "../../../sdkwork-specs/tools/check-database-framework-standard.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("canonical database framework validator accepts the Agents contract", () => {
  const result = validateDatabaseFramework(repoRoot);

  assert.equal(result.skipped, false);
  assert.deepEqual(result.failures, []);
  assert.equal(result.ok, true);
});

test("agents database manifest declares one canonical PostgreSQL engine", () => {
  const manifest = JSON.parse(
    readFileSync(path.join(repoRoot, "database/database.manifest.json"), "utf8"),
  );
  assert.equal(manifest.schemaVersion, 2);
  assert.equal(manifest.databaseRole, "authoritative-server");
  assert.equal(manifest.moduleId, "agents");
  assert.deepEqual(manifest.engines, ["postgres"]);
  assert.equal(manifest.defaultEngine, "postgres");
  assert.equal(manifest.tablePrefix, "ai_");
  assert.equal(manifest.contractVersion, "7.2.0");
  assert.equal(manifest.baselineStrategy, "baseline-plus-migrations");
  assert.equal(
    manifest.lifecycle.autoMigrate,
    false,
    "authoritative startup must not execute pending migrations implicitly",
  );
  assert.equal(
    manifest.baselineAnchorTable,
    "ai_agent_tool_asset",
    "the pre-launch completion anchor must be created only after the complete Agents baseline",
  );
  assert.equal(manifest.paths.contract, "contract/schema.yaml");
  assert.equal(
    existsSync(
      path.join(repoRoot, "database/ddl/baseline/sqlite/0001_agents_baseline.sql"),
    ),
    false,
    "Agents must not retain a partial SQLite schema beside its PostgreSQL authority",
  );
  assert.equal(
    existsSync(path.join(repoRoot, "database/migrations/sqlite/README.md")),
    false,
    "Agents must not advertise an inactive SQLite migration path",
  );
});

test("agents database contract is materialized without placeholders", () => {
  const schemaPath = path.join(repoRoot, "database/contract/schema.yaml");
  const schema = readFileSync(schemaPath, "utf8");
  assert.doesNotMatch(schema, /<module-id>/);
  assert.match(schema, /table_prefix: ai_/u);
  assert.match(schema, /contract_version: 7\.2\.0/u);
  assert.match(
    schema,
    /ddl_authority: ddl\/baseline\/postgres\/0001_agents_baseline\.sql/u,
  );
  assert.equal((schema.match(/lifecycle_status: expanding/gu) ?? []).length, 0);
  assert.equal((schema.match(/lifecycle_status: active/gu) ?? []).length, 30);
  assert.equal(
    (schema.match(/- \[document, documents\]/gu) ?? []).length,
    2,
    "agent and project composition contracts must both declare document/documents",
  );
  assert.match(
    schema,
    /- module: sdkwork-documents\s+columns: \[target_ref, target_version_ref\]/u,
    "Documents must be declared as an external reference owner",
  );

  const registry = JSON.parse(
    readFileSync(path.join(repoRoot, "database/contract/table-registry.json"), "utf8"),
  );
  assert.equal(registry.contractVersion, "7.2.0");
  assert.equal(registry.tables.length, 30);
  assert.ok(
    registry.tables.every((entry) => entry.lifecycle_status === "active"),
    "every Agents 7.2 table must be active in the contract registry",
  );
});

test("PostgreSQL baseline exactly matches the 30-table contract registry", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const baselineTables = Array.from(
    baseline.matchAll(/CREATE TABLE IF NOT EXISTS (ai_[a-z_]+)/giu),
    (match) => match[1],
  );
  const registry = JSON.parse(
    readFileSync(path.join(repoRoot, "database/contract/table-registry.json"), "utf8"),
  );
  const registryTables = registry.tables.map((entry) => entry.table_name);
  const completionAnchor = JSON.parse(
    readFileSync(path.join(repoRoot, "database/database.manifest.json"), "utf8"),
  ).baselineAnchorTable;

  assert.deepEqual(
    [...new Set(baselineTables)].sort(),
    [...registryTables].sort(),
  );
  assert.equal(
    baselineTables.at(-1),
    completionAnchor,
    "baselineAnchorTable must remain the final table created by the atomic greenfield baseline",
  );
  assert.equal(
    registryTables.at(-1),
    completionAnchor,
    "the table registry must end with the pre-launch baseline completion anchor",
  );
  assert.doesNotMatch(
    baseline,
    /\bai_(?:coding_session|agent_message|agent_chat_turn|chat_conversation|chat_message)\b/iu,
  );
  assert.doesNotMatch(baseline, /\bim_[a-z_]+\b/iu);
});

test("agent project and execution table names close across every persistence authority", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const schema = readFileSync(
    path.join(repoRoot, "database/contract/schema.yaml"),
    "utf8",
  );
  const persistence = [
    "crates/sdkwork-intelligence-agents-service/src/persistence.rs",
    "crates/sdkwork-intelligence-agents-service/src/persistence/sql.rs",
  ]
    .map((relativePath) => readFileSync(path.join(repoRoot, relativePath), "utf8"))
    .join("\n");
  const registry = JSON.parse(
    readFileSync(path.join(repoRoot, "database/contract/table-registry.json"), "utf8"),
  );
  const registeredTables = new Set(
    registry.tables.map((entry) => entry.table_name),
  );
  const aggregateTables = [
    "ai_agent_workspace",
    "ai_agent_project",
    "ai_agent_project_composition_slot",
    "ai_agent_session",
    "ai_agent_session_runtime_binding",
    "ai_agent_turn",
    "ai_agent_session_item",
    "ai_agent_interaction",
    "ai_agent_session_checkpoint",
    "ai_agent_composition_slot",
  ];

  assert.ok(
    registry.tables.every(
      (entry) =>
        entry.table_name === "ai_agent" || entry.table_name.startsWith("ai_agent_"),
    ),
    "Agents registry must use only the registered ai_agent physical namespace",
  );
  for (const tableName of aggregateTables) {
    assert.ok(registeredTables.has(tableName), `${tableName} must be registered`);
    assert.match(
      baseline,
      new RegExp(`CREATE TABLE IF NOT EXISTS ${tableName}\\s*\\(`, "u"),
    );
    assert.match(schema, new RegExp(`^  - name: ${tableName}$`, "mu"));
    assert.match(
      persistence,
      new RegExp(`\\b(?:FROM|INTO|UPDATE)\\s+${tableName}\\b`, "u"),
      `${tableName} must be used by the Rust persistence authority`,
    );
  }
});

test("database materialization preserves authored semantic metadata and is stable", () => {
  const tempBase = path.resolve(os.tmpdir());
  const stagingRoot = mkdtempSync(
    path.join(tempBase, "sdkwork-agents-db-materializer-test-"),
  );
  const databaseRoot = path.join(stagingRoot, "database");
  const contractFiles = [
    "database.manifest.json",
    "contract/prefix-registry.json",
    "contract/schema.yaml",
    "contract/table-registry.json",
  ];

  try {
    cpSync(path.join(repoRoot, "database"), databaseRoot, { recursive: true });
    const before = Object.fromEntries(
      contractFiles.map((relativePath) => [
        relativePath,
        readFileSync(path.join(databaseRoot, relativePath), "utf8"),
      ]),
    );

    for (let attempt = 0; attempt < 2; attempt += 1) {
      const result = spawnSync(
        process.execPath,
        [
          path.join(
            repoRoot,
            "tools/database/materialize-agents-database-contract.mjs",
          ),
          "--root",
          stagingRoot,
        ],
        { cwd: repoRoot, encoding: "utf8" },
      );
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    }

    for (const relativePath of contractFiles) {
      assert.equal(
        readFileSync(path.join(databaseRoot, relativePath), "utf8"),
        before[relativePath],
        `${relativePath} must remain byte-stable after materialization`,
      );
    }

    const registry = JSON.parse(before["contract/table-registry.json"]);
    assert.ok(
      registry.tables.every(
        (entry) =>
          entry.profile &&
          entry.write_owner &&
          entry.system_of_record === true &&
          entry.compliance_level &&
          entry.lifecycle_status,
      ),
      "materialization must preserve authored table ownership and profile metadata",
    );
  } finally {
    assert.equal(path.dirname(stagingRoot), tempBase);
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test("composition slot enums and canonical module pairs stay aligned", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const expectedPairs = [
    ["memory", "memory"],
    ["knowledge", "knowledgebase"],
    ["skill", "skills"],
    ["prompt", "prompts"],
    ["drive", "drive"],
    ["document", "documents"],
    ["tool", "tools"],
    ["mcp", "mcp"],
  ];

  for (const contract of [
    {
      table: "ai_agent_composition_slot",
      kind: "ck_ai_agent_composition_slot_kind",
      module: "ck_ai_agent_composition_slot_module",
      pair: "ck_ai_agent_composition_slot_pair",
    },
    {
      table: "ai_agent_project_composition_slot",
      kind: "ck_ai_agent_project_slot_kind",
      module: "ck_ai_agent_project_slot_module",
      pair: "ck_ai_agent_project_slot_pair",
    },
  ]) {
    const tableSql = extractCreateTableSql(baseline, contract.table);
    const kindValues = extractSingleColumnCheckValues(tableSql, contract.kind);
    const moduleValues = extractSingleColumnCheckValues(tableSql, contract.module);
    const pairs = extractPairCheckValues(tableSql, contract.pair);

    assert.deepEqual(new Set(kindValues), new Set(expectedPairs.map(([kind]) => kind)));
    assert.deepEqual(
      new Set(moduleValues),
      new Set(expectedPairs.map(([, module]) => module)),
    );
    assert.deepEqual(new Set(pairs), new Set(expectedPairs.map((pair) => pair.join("/"))));
  }
});

test("pre-launch PostgreSQL contract is consolidated on the folded baseline", () => {
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const activeMigrations = readdirSync(migrationRoot)
    .filter((fileName) => /\.(?:up|down)\.sql$/u.test(fileName))
    .sort();
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const canonicalLineageColumns = [
    "provider_session_id",
    "provider_session_tree_id",
    "provider_parent_session_id",
    "provider_forked_from_session_id",
  ];

  assert.deepEqual(
    activeMigrations,
    [],
    "pre-launch agents schema must be installed solely from the folded baseline",
  );
  for (const canonicalName of canonicalLineageColumns) {
    assert.match(baseline, new RegExp(`\\b${canonicalName}\\s+VARCHAR\\(256\\)`, "u"));
  }
  assert.match(
    baseline,
    /CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_session_runtime_binding_provider_session/iu,
  );
  const interactionSql = extractCreateTableSql(baseline, "ai_agent_interaction");
  assert.match(interactionSql, /request_json\s+JSONB,/iu);
  assert.match(
    interactionSql,
    /CONSTRAINT\s+ck_ai_agent_interaction_kind\s+CHECK\s*\(kind\s+IN\s*\(0,\s*1,\s*2,\s*3\)\)/iu,
  );
  assert.match(
    interactionSql,
    /jsonb_typeof\(request_json\)\s*=\s*'object'/iu,
  );
});

test("pre-launch PostgreSQL migrations stay empty until the baseline evolves", () => {
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const upMigrations = readdirSync(migrationRoot).filter((fileName) =>
    fileName.endsWith(".up.sql"),
  );
  assert.deepEqual(
    upMigrations,
    [],
    "pre-launch agents schema must not carry post-baseline migrations; fold them into the baseline",
  );
});

test("baseline outbox dedupe is tenant and organization scoped", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  assert.match(
    baseline,
    /CONSTRAINT\s+uk_ai_agent_outbox_event_dedupe\s+UNIQUE\s*\(\s*tenant_id,\s*organization_id,\s*dedupe_key\s*\)/iu,
  );
});

test("provider session identity stays normalized and unique for its full lifecycle", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const tableSql = extractCreateTableSql(
    baseline,
    "ai_agent_session_runtime_binding",
  );

  assert.match(
    tableSql,
    /provider_session_id\s+IS\s+NULL\s+OR\s*\(\s*provider_session_id\s+<>\s+''\s+AND\s+provider_session_id\s*=\s*BTRIM\(provider_session_id\)/iu,
  );
  assert.match(
    baseline,
    /CREATE\s+UNIQUE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+uk_ai_agent_session_runtime_binding_provider_session[\s\S]*?\(\s*tenant_id,\s*organization_id,\s*owner_user_id,\s*provider_binding_id,\s*provider_id,\s*provider_session_id\s*\)\s+WHERE\s+provider_session_id\s+IS\s+NOT\s+NULL\s*;/iu,
  );
  assert.doesNotMatch(
    baseline,
    /uk_ai_agent_session_runtime_binding_provider_session[\s\S]*?status\s*<>\s*3/iu,
  );
});

test("resource user-state baseline uses the canonical last-read item sequence", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const tableSql = extractCreateTableSql(
    baseline,
    "ai_agent_resource_user_state",
  );

  assert.match(tableSql, /last_read_item_sequence\s+BIGINT/iu);
  assert.doesNotMatch(tableSql, /last_seen_item_sequence/iu);
});

test("audit baseline accepts every action emitted by the runtime", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const domain = readFileSync(
    path.join(
      repoRoot,
      "crates/sdkwork-intelligence-agents-service/src/domain.rs",
    ),
    "utf8",
  );
  const actionCodes = Array.from(
    domain
      .match(/pub fn action_code[\s\S]*?\r?\n    \}\r?\n\}/u)?.[0]
      ?.matchAll(/=>\s*"([a-z_]+)"/gu) ?? [],
    (match) => match[1],
  );

  assert.ok(actionCodes.length > 0, "runtime audit action codes must be discoverable");
  for (const actionCode of actionCodes) {
    assert.match(
      baseline,
      new RegExp(`'${actionCode}'`, "u"),
      `audit constraint must accept runtime action ${actionCode}`,
    );
  }
});

test("audit baseline supports non-agent aggregates without fabricated agent keys", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  assert.match(baseline, /aggregate_type\s+VARCHAR\(64\)\s+NOT NULL/iu);
  assert.match(baseline, /agent_internal_id\s+BIGINT,/iu);
  assert.match(baseline, /agent_id\s+VARCHAR\(128\),/iu);
  assert.match(baseline, /aggregate_type\s+<>\s+'agent'/iu);
});

function extractCreateTableSql(baseline, tableName) {
  const start = baseline.indexOf(`CREATE TABLE IF NOT EXISTS ${tableName} (`);
  assert.notEqual(start, -1, `${tableName} must exist in the PostgreSQL baseline`);
  const end = baseline.indexOf("\n);", start);
  assert.notEqual(end, -1, `${tableName} DDL must have a closing delimiter`);
  return baseline.slice(start, end + 3);
}

function extractConstraintSql(tableSql, constraintName) {
  const start = tableSql.indexOf(`CONSTRAINT ${constraintName} CHECK (`);
  assert.notEqual(start, -1, `${constraintName} must exist`);
  const nextConstraint = tableSql.indexOf("\n    CONSTRAINT ", start + 1);
  return tableSql.slice(start, nextConstraint === -1 ? tableSql.length : nextConstraint);
}

function extractSingleColumnCheckValues(tableSql, constraintName) {
  return Array.from(
    extractConstraintSql(tableSql, constraintName).matchAll(/'([^']+)'/gu),
    (match) => match[1],
  );
}

function extractPairCheckValues(tableSql, constraintName) {
  return Array.from(
    extractConstraintSql(tableSql, constraintName).matchAll(
      /\('([^']+)',\s*'([^']+)'\)/gu,
    ),
    (match) => `${match[1]}/${match[2]}`,
  );
}

const databaseContractDocumentationFiles = [
  "database/README.md",
  "database/SCHEMA_DESIGN.md",
  "database/MIGRATION_SUMMARY.md",
];

const forbiddenDatabaseContractPhrases = [
  /\bpost-GA\b/iu,
  /\bunimplemented\b/iu,
  /\bdeferred until\b/iu,
  /\buntil\b[\s\S]{0,80}\bimplemented\b/iu,
  /\bTBD\b/iu,
];

test("database contract docs describe current scope without deferred debt wording", () => {
  for (const relativePath of databaseContractDocumentationFiles) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    for (const pattern of forbiddenDatabaseContractPhrases) {
      assert.doesNotMatch(
        content,
        pattern,
        `${relativePath} must describe current database scope and entry criteria instead of deferred or unimplemented work`,
      );
    }
  }
});
