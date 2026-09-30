#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];

function readText(relativePath) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(absolutePath)) {
    failures.push(`${relativePath} must exist`);
    return '';
  }
  return fs.readFileSync(absolutePath, 'utf8');
}

function readJson(relativePath) {
  const text = readText(relativePath).replace(/^\uFEFF/, '');
  return JSON.parse(text);
}

function assert(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

function assertDirectory(relativePath) {
  assert(fs.existsSync(path.join(repoRoot, relativePath)), `${relativePath}/ must exist`);
}

function toRepoRelative(absolutePath) {
  return path.relative(repoRoot, absolutePath).replaceAll('\\', '/');
}

function listAuthoredFiles(dir, predicate, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (
      entry.name === 'node_modules'
      || entry.name === 'target'
      || entry.name === 'dist'
      || entry.name === 'build'
      || entry.name === 'coverage'
      || entry.name === '.git'
    ) {
      continue;
    }
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      listAuthoredFiles(absolute, predicate, files);
    } else if (predicate(absolute)) {
      files.push(absolute);
    }
  }
  return files;
}

const requiredDirectories = [
  'apis',
  'apps',
  'crates',
  'sdks',
  'database',
  'deployments',
  'etc',
  'scripts',
  'docs',
  'tests',
  '.sdkwork',
  'specs',
];

for (const directory of requiredDirectories) {
  assertDirectory(directory);
}

assert(fs.existsSync(path.join(repoRoot, 'sdkwork.app.config.json')), 'sdkwork.app.config.json must exist');
assert(fs.existsSync(path.join(repoRoot, 'sdkwork.workflow.json')), 'sdkwork.workflow.json must exist');
assert(fs.existsSync(path.join(repoRoot, 'package.json')), 'package.json must exist');
assert(
  fs.existsSync(path.join(repoRoot, '.github/workflows/package.yml')),
  '.github/workflows/package.yml must exist per GITHUB_WORKFLOW_SPEC.md',
);

const packageJson = readJson('package.json');
for (const script of ['dev', 'build', 'test', 'check', 'verify', 'clean']) {
  assert(packageJson.scripts?.[script], `package.json must expose pnpm ${script}`);
}
assert(packageJson.scripts?.['check:architecture-alignment'], 'package.json must expose check:architecture-alignment');
assert(packageJson.scripts?.['topology:validate'], 'package.json must expose topology:validate');
assert(packageJson.scripts?.['check:api-envelope'], 'package.json must expose check:api-envelope');
assert(packageJson.scripts?.['check:deploy'], 'package.json must expose check:deploy');
assert(packageJson.scripts?.['check:docs'], 'package.json must expose check:docs');
assert(packageJson.dependencies?.['@sdkwork/app-topology'], 'package.json must declare @sdkwork/app-topology');

const cargoToml = readText('Cargo.toml');
assert(cargoToml.includes('sdkwork-web-core'), 'Cargo.toml must declare sdkwork-web-core');
assert(cargoToml.includes('sdkwork-web-axum'), 'Cargo.toml must declare sdkwork-web-axum');
assert(cargoToml.includes('sdkwork-iam-web-adapter'), 'Cargo.toml must declare sdkwork-iam-web-adapter');
assert(cargoToml.includes('sdkwork-database-config'), 'Cargo.toml must declare sdkwork-database-config');
assert(cargoToml.includes('sdkwork-database-sqlx'), 'Cargo.toml must declare sdkwork-database-sqlx');
assert(cargoToml.includes('sdkwork-utils-rust'), 'Cargo.toml must declare sdkwork-utils-rust');
assert(cargoToml.includes('sdkwork-agent-server'), 'Cargo.toml must declare sdkwork-kernel sdkwork-agent-server dependency');
assert(cargoToml.includes('sdkwork-intelligence-agents-service'), 'Cargo.toml must declare sdkwork-intelligence-agents-service');
assert(cargoToml.includes('sdkwork-routes-agents-http-shared'), 'Cargo.toml must declare sdkwork-routes-agents-http-shared');
assert(!cargoToml.includes('sdkwork-agent-business'), 'Cargo.toml must not reference retired sdkwork-agent-business');
assert(!cargoToml.includes('sdkwork-discovery'), 'sdkwork-discovery is deferred until RPC services exist');

function assertNoForbiddenCompositionManifests(dir, relativePrefix = '') {
  if (!fs.existsSync(dir)) return;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name === 'target' || entry.name === '.git') continue;
    const absolute = path.join(dir, entry.name);
    const relative = relativePrefix ? `${relativePrefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) {
      assertNoForbiddenCompositionManifests(absolute, relative);
      continue;
    }
    assert(
      entry.name !== 'dependency.composition.json',
      `${relative} is forbidden; use core component.spec.json#contracts.sdkDependencies per APP_COMPOSITION_SPEC.md`,
    );
  }
}
assertNoForbiddenCompositionManifests(repoRoot);

const contractSource = readText('crates/sdkwork-agents-contract/src/lib.rs');
assert(
  contractSource.includes('sdkwork_utils_rust::parse_bool'),
  'sdkwork-agents-contract must use sdkwork-utils-rust parse_bool',
);

const bridgeSource = readText('crates/sdkwork-agents-kernel-bridge/src/lib.rs');
const agentHttpStateSource = readText('crates/sdkwork-agents-kernel-bridge/src/agent_http_state.rs');
assert(
  bridgeSource.includes('build_served_combined_router'),
  'kernel-bridge must compose kernel web-framework served router',
);
assert(
  bridgeSource.includes('app::build_app'),
  'kernel-bridge must compose kernel operational router',
);
assert(
  bridgeSource.includes('build_agent_http_state'),
  'kernel-bridge must bootstrap agent HTTP state through dedicated module',
);
assert(
  agentHttpStateSource.includes('connect_from_agents_database_env'),
  'agent_http_state must wire the canonical Agents PostgreSQL database in production',
);
assert(
  agentHttpStateSource.includes('agents_use_dev_inline_auth_resolver'),
  'agent_http_state must gate dev inline auth through contract helpers',
);

assert(fs.existsSync(path.join(repoRoot, 'crates/sdkwork-agents-database-host/src/lib.rs')), 'sdkwork-agents-database-host crate must exist');
assert(fs.existsSync(path.join(repoRoot, 'crates/sdkwork-api-agents-assembly/src/lib.rs')), 'sdkwork-api-agents-assembly crate must exist');
assert(fs.existsSync(path.join(repoRoot, 'crates/sdkwork-agents-integration-tests/Cargo.toml')), 'integration-tests crate must exist');
assert(fs.existsSync(path.join(repoRoot, 'deployments/deploy.yaml')), 'deployments/deploy.yaml must exist per SDKWORK_DEPLOY_SPEC.md');
assert(fs.existsSync(path.join(repoRoot, 'deployments/docker/Dockerfile')), 'deployments/docker/Dockerfile must exist');
assert(fs.existsSync(path.join(repoRoot, '.env.example')), '.env.example must exist');

const apiServerCargo = readText('crates/sdkwork-api-agents-standalone-gateway/Cargo.toml');
assert(
  apiServerCargo.includes('sdkwork-api-agents-standalone-gateway'),
  'standalone-gateway must expose sdkwork-api-agents-standalone-gateway binary alias',
);

const topologySpec = readJson('specs/topology.spec.json');
assert(
  topologySpec.scripts?.dev === 'scripts/agents-dev.mjs',
  'topology.spec.json dev script must reference scripts/agents-dev.mjs',
);
assert(
  !JSON.stringify(topologySpec).includes('llm-dev.mjs'),
  'topology.spec.json must not retain llm stale script references',
);
assert(
  topologySpec.components?.cloudGateway === undefined,
  'topology.spec.json must not declare an application-local cloud gateway component',
);

for (const profileFile of Object.values(topologySpec.profileFiles ?? {})) {
  assert(fs.existsSync(path.join(repoRoot, profileFile)), `${profileFile} must exist for topology profile`);
}

const boundaries = readText('crates/sdkwork-agents-kernel-bridge/src/boundaries.rs');
assert(boundaries.includes('agents-domain-service'), 'boundaries must declare agents-domain-service capability');
assert(!boundaries.includes('business'), 'boundaries must not retain business capability tokens');

for (const sdkManifestPath of [
  'sdks/sdkwork-agents-sdk/sdk-manifest.json',
  'sdks/sdkwork-agents-app-sdk/sdk-manifest.json',
  'sdks/sdkwork-agents-backend-sdk/sdk-manifest.json',
]) {
  const sdkManifest = readJson(sdkManifestPath);
  assert(sdkManifest.sdkOwner === 'sdkwork-agents', `${sdkManifestPath} sdkOwner must be sdkwork-agents`);
}

const kernelRoot = path.resolve(repoRoot, '..', 'sdkwork-kernel');
const retiredKernelPaths = [
  'sdkwork-agent-business',
  'apis/agent-business',
  'crates/sdkwork-routes-agent-http-shared',
  'crates/sdkwork-routes-agent-open-api',
  'crates/sdkwork-routes-agent-app-api',
  'crates/sdkwork-routes-agent-backend-api',
  'sdks/sdkwork-agent-sdk',
  'sdks/sdkwork-agent-app-sdk',
  'sdks/sdkwork-agent-backend-sdk',
];
for (const retiredPath of retiredKernelPaths) {
  assert(
    !fs.existsSync(path.join(kernelRoot, retiredPath)),
    `sdkwork-kernel must not retain retired path ${retiredPath}; managed agents domain belongs in sdkwork-agents`,
  );
}

const componentSpec = readJson('specs/component.spec.json');
const sdkDeps = componentSpec.contracts?.sdkDependencies ?? [];
const depIds = sdkDeps.map((entry) => entry.workspace);
assert(depIds.includes('sdkwork-web-framework'), 'component.spec.json must declare sdkwork-web-framework');
assert(depIds.includes('sdkwork-database'), 'component.spec.json must declare sdkwork-database');
assert(depIds.includes('sdkwork-utils'), 'component.spec.json must declare sdkwork-utils');
assert(depIds.includes('sdkwork-kernel'), 'component.spec.json must declare sdkwork-kernel');
assert(
  !depIds.some((workspace) => /^sdkwork-im(?:$|-)/u.test(workspace)),
  'component.spec.json must not declare sdkwork-im; sdkwork-im is an Agents consumer',
);
const driveDep = sdkDeps.find((entry) => entry.workspace === 'sdkwork-drive-app-sdk');
assert(driveDep, 'component.spec.json must declare sdkwork-drive for upload integration');
assert(!depIds.includes('sdkwork-discovery'), 'component.spec.json must not require sdkwork-discovery yet');

const independentCapabilityModules = [
  ['sdkwork-memory-app-sdk', 'composition-slot', 'memory', 'memory'],
  ['sdkwork-knowledgebase-app-sdk', 'composition-slot', 'knowledge', 'knowledgebase'],
  ['sdkwork-skills-app-sdk', 'composition-slot', 'skill', 'skills'],
  ['sdkwork-prompts-app-sdk', 'composition-slot', 'prompt', 'prompts'],
  ['sdkwork-documents-app-sdk', 'composition-slot', 'document', 'documents'],
  ['sdkwork-mcp-app-sdk', 'composition-slot', 'mcp', 'mcp'],
  ['sdkwork-llm-app-sdk', 'runtime-binding-provider-profile', null, null],
  ['sdkwork-drive-app-sdk', 'composition-slot', 'drive', 'drive'],
];
for (const [workspace, integrationMode, slotKind, targetModule] of independentCapabilityModules) {
  const dep = sdkDeps.find((entry) => entry.workspace === workspace);
  assert(dep, `component.spec.json must declare ${workspace} as an independent capability module`);
  assert(
    dep?.dependencyMode === 'independent-capability-module',
    `component.spec.json ${workspace} dependencyMode must be independent-capability-module`,
  );
  assert(
    dep?.integrationMode === integrationMode,
    `component.spec.json ${workspace} integrationMode must be ${integrationMode}`,
  );
  assert(
    dep?.reverseDependencyPolicy === 'forbidden',
    `component.spec.json ${workspace} reverseDependencyPolicy must be forbidden`,
  );
  if (integrationMode === 'composition-slot') {
    assert(
      dep?.slotKind === slotKind,
      `component.spec.json ${workspace} slotKind must be ${slotKind}`,
    );
    assert(
      dep?.targetModule === targetModule,
      `component.spec.json ${workspace} targetModule must be ${targetModule}`,
    );
  }
}

const appManifest = readJson('sdkwork.app.config.json');
assert(
  appManifest.metadata?.topologySpec === 'specs/topology.spec.json',
  'sdkwork.app.config.json metadata.topologySpec must reference specs/topology.spec.json',
);
assert(
  appManifest.metadata?.kernelDependency?.includes('sdkwork-kernel'),
  'sdkwork.app.config.json must declare kernel dependency',
);
assert(
  appManifest.app?.appType === 'NONE',
  'sdkwork.app.config.json appType must use canonical PlusProjectType NONE for a server-only application',
);

const workspaceYaml = readText('pnpm-workspace.yaml');
assert(
  workspaceYaml.includes('knowledgebase-app-sdk'),
  'pnpm-workspace.yaml must declare sdkwork-knowledgebase-app-sdk for PC/H5 composition',
);
assert(
  workspaceYaml.includes('sdkwork-utils-typescript'),
  'pnpm-workspace.yaml must declare @sdkwork/utils for iam-contracts transitive workspace resolution',
);

const agentsImBoundarySpec = readText('specs/AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md');
assert(
  agentsImBoundarySpec.includes('sdkwork-im -> sdkwork-agents -> sdkwork-kernel'),
  'Agents IM boundary spec must declare sdkwork-im -> sdkwork-agents -> sdkwork-kernel',
);
assert(
  agentsImBoundarySpec.includes('Agents never depends on IM'),
  'Agents IM boundary spec must forbid the reverse sdkwork-agents-to-sdkwork-im dependency',
);

const agentsDatabaseSpec = readText(
  'crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md',
);
const agentsTableRegistry = readJson('database/contract/table-registry.json');
const agentTableNames = (agentsTableRegistry.tables ?? []).map((table) => table.table_name);
for (const targetTable of [
  'ai_agent_workspace',
  'ai_agent_project',
  'ai_agent_project_composition_slot',
  'ai_agent_session',
  'ai_agent_session_runtime_binding',
  'ai_agent_turn',
  'ai_agent_turn_input_queue_entry',
  'ai_agent_session_item',
  'ai_agent_item_drive_ref',
  'ai_agent_item_feedback',
  'ai_agent_interaction',
  'ai_agent_session_checkpoint',
  'ai_agent_task',
  'ai_agent_task_run',
  'ai_agent_task_run_attempt',
  'ai_agent_resource_user_state',
  'ai_agent_project_member',
  'ai_agent_share_link',
  'ai_agent_outbox_event',
]) {
  assert(
    agentsDatabaseSpec.includes(`\`${targetTable}\``) && agentTableNames.includes(targetTable),
    `Agents database specification and registry must define ${targetTable}`,
  );
}
assert(
  agentsTableRegistry.contractVersion === '7.2.0'
    && agentTableNames.length === 30
    && agentsDatabaseSpec.includes('owns exactly 30 tables'),
  'Agents database contract must declare the active 30-table inventory',
);
for (const retiredTable of [
  'ai_agent_chat_turn',
  'ai_agent_message_drive_ref',
  'ai_agent_message_feedback',
]) {
  assert(
    !agentTableNames.includes(retiredTable) && !agentsDatabaseSpec.includes(`\`${retiredTable}\``),
    `Agents must not retain the retired parallel chat/message table ${retiredTable}`,
  );
}
assert(
  agentsDatabaseSpec.includes('There are no cross-module foreign keys')
    && agentsDatabaseSpec.includes('does not expose a generic publisher SPI')
    && agentsDatabaseSpec.includes('must not add a local Kafka'),
  'Agents must keep downstream indexing, generation and communication capabilities independent',
);

const forbiddenImDependencyPattern = /(?:@sdkwork\/im-|sdkwork[-_]im(?:[-_][a-z0-9_-]+)?|\.\.[\\/]sdkwork-im)(?:[\\/]|\b)/iu;
for (const [manifestPath, manifestText] of [
  ['Cargo.toml', cargoToml],
  ['package.json', JSON.stringify(packageJson)],
  ['pnpm-workspace.yaml', workspaceYaml],
  ['specs/component.spec.json', JSON.stringify(componentSpec.contracts?.sdkDependencies ?? [])],
]) {
  assert(
    !forbiddenImDependencyPattern.test(manifestText),
    `${manifestPath} must not declare an sdkwork-im dependency`,
  );
}

for (const manifestPath of listAuthoredFiles(
  repoRoot,
  (candidate) => ['Cargo.toml', 'package.json'].includes(path.basename(candidate)),
)) {
  const relativePath = toRepoRelative(manifestPath);
  if (path.basename(manifestPath) === 'package.json') {
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    for (const dependencyGroup of [
      'dependencies',
      'devDependencies',
      'optionalDependencies',
      'peerDependencies',
    ]) {
      for (const dependencyName of Object.keys(manifest[dependencyGroup] ?? {})) {
        assert(
          !forbiddenImDependencyPattern.test(dependencyName),
          `${relativePath} ${dependencyGroup} must not declare ${dependencyName}; sdkwork-im is a consumer`,
        );
      }
    }
  } else {
    const manifest = fs.readFileSync(manifestPath, 'utf8');
    assert(
      !forbiddenImDependencyPattern.test(manifest),
      `${relativePath} must not declare an sdkwork-im crate or path dependency`,
    );
  }
}

const sourceImportPattern = /(?:\bfrom\s*|\bimport\s*\(|\brequire\s*\(|\buse\s+|\bextern\s+crate\s+|\bpath\s*=\s*)['"]?(?:@sdkwork\/im-[a-z0-9_./-]*|sdkwork_im_[a-z0-9_]*|\.\.[\\/]sdkwork-im(?:[\\/][a-z0-9_./\\-]*)?)/iu;
const authoredSourceExtensions = /\.(?:cjs|js|jsx|mjs|rs|ts|tsx)$/u;
for (const sourceRoot of ['apis', 'apps', 'crates', 'sdks']) {
  for (const filePath of listAuthoredFiles(path.join(repoRoot, sourceRoot), (candidate) => authoredSourceExtensions.test(candidate))) {
    const source = fs.readFileSync(filePath, 'utf8');
    assert(
      !sourceImportPattern.test(source),
      `${toRepoRelative(filePath)} must not import sdkwork-im; dependency direction is sdkwork-im -> sdkwork-agents`,
    );
  }
}

const imSqlOwnershipPattern = /\b(?:CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?|ALTER\s+TABLE\s+|INSERT\s+INTO\s+|UPDATE\s+|DELETE\s+FROM\s+|REFERENCES\s+)['"`]?im_[a-z0-9_]+/iu;
const imContractTablePattern = /(?:\bname\s*:\s*|"(?:table|tableName|physicalName)"\s*:\s*")im_[a-z0-9_]+/iu;
for (const databaseRoot of ['database', 'crates']) {
  for (const filePath of listAuthoredFiles(
    path.join(repoRoot, databaseRoot),
    (candidate) => /\.(?:json|sql|ya?ml)$/u.test(candidate) && !candidate.includes(`${path.sep}specs${path.sep}`),
  )) {
    const source = fs.readFileSync(filePath, 'utf8');
    assert(
      !imSqlOwnershipPattern.test(source) && !imContractTablePattern.test(source),
      `${toRepoRelative(filePath)} must not declare, mutate, or reference an im_* table`,
    );
  }
}

for (const sourceRoot of ['apis', 'apps', 'crates', 'sdks']) {
  for (const filePath of listAuthoredFiles(
    path.join(repoRoot, sourceRoot),
    (candidate) => /\.(?:cjs|js|jsx|mjs|rs|ts|tsx)$/u.test(candidate),
  )) {
    assert(
      !imSqlOwnershipPattern.test(fs.readFileSync(filePath, 'utf8')),
      `${toRepoRelative(filePath)} must not contain SQL that mutates or references an im_* table`,
    );
  }
}

const forbiddenCapabilitySdkImports = ['@sdkwork/agents-app-sdk', '@sdkwork/knowledgebase-app-sdk'];
function listTypeScriptSources(dir, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name === 'dist') continue;
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      listTypeScriptSources(absolute, files);
      continue;
    }
    if (/\.(?:ts|tsx)$/u.test(entry.name) && !entry.name.endsWith('.d.ts')) {
      files.push(absolute);
    }
  }
  return files;
}

for (const capabilityRoot of [
  'apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents',
  'apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents',
]) {
  const capabilityPackageJsonPath = path.join(repoRoot, capabilityRoot, 'package.json');
  if (fs.existsSync(capabilityPackageJsonPath)) {
    const capabilityPackageJson = JSON.parse(fs.readFileSync(capabilityPackageJsonPath, 'utf8'));
    for (const specifier of forbiddenCapabilitySdkImports) {
      if (capabilityPackageJson.dependencies?.[specifier]) {
        failures.push(
          `${path.relative(repoRoot, capabilityPackageJsonPath).replaceAll('\\', '/')}: remove direct ${specifier} dependency; consume through *-core/sdk`,
        );
      }
    }
  }

  for (const filePath of listTypeScriptSources(path.join(repoRoot, capabilityRoot))) {
    const source = fs.readFileSync(filePath, 'utf8');
    for (const specifier of forbiddenCapabilitySdkImports) {
      if (source.includes(`from "${specifier}"`) || source.includes(`from '${specifier}'`)) {
        failures.push(
          `${path.relative(repoRoot, filePath).replaceAll('\\', '/')}: capability package must import ${specifier} through *-core/sdk only`,
        );
      }
    }
  }
}

for (const appShellRoot of ['apps/sdkwork-agents-pc', 'apps/sdkwork-agents-h5']) {
  const appShellPackageJsonPath = path.join(repoRoot, appShellRoot, 'package.json');
  if (fs.existsSync(appShellPackageJsonPath)) {
    const appShellPackageJson = JSON.parse(fs.readFileSync(appShellPackageJsonPath, 'utf8'));
    for (const specifier of forbiddenCapabilitySdkImports) {
      if (appShellPackageJson.dependencies?.[specifier]) {
        failures.push(
          `${path.relative(repoRoot, appShellPackageJsonPath).replaceAll('\\', '/')}: app shell must not declare ${specifier}; consume through *-core/sdk`,
        );
      }
    }
  }

  for (const scanRoot of ['src', 'scripts']) {
    for (const filePath of listTypeScriptSources(path.join(repoRoot, appShellRoot, scanRoot))) {
      const source = fs.readFileSync(filePath, 'utf8');
      for (const specifier of forbiddenCapabilitySdkImports) {
        if (source.includes(`from "${specifier}"`) || source.includes(`from '${specifier}'`)) {
          failures.push(
            `${path.relative(repoRoot, filePath).replaceAll('\\', '/')}: app shell must import ${specifier} through *-core/sdk only`,
          );
        }
      }
    }
  }
}

if (failures.length > 0) {
  console.error('sdkwork-agents architecture alignment failures:');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log('sdkwork-agents architecture alignment passed');
