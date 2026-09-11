#!/usr/bin/env node
// Agent structured-call contract checker.
//
// Validates that specs/agent-structured-call.contract.json stays aligned with
// the normative spec, the OpenAPI wire authority, the Rust implementation,
// and the TypeScript app SDK. Exits non-zero on the first violation.
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const contractPath = path.join(repoRoot, 'specs', 'agent-structured-call.contract.json');

function fail(message) {
  console.error(`agent-structured-call check failed: ${message}`);
  process.exit(1);
}

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) fail(`${label} is missing: ${filePath}`);
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    fail(`${label} is not valid JSON (${filePath}): ${error.message}`);
  }
}

function readText(filePath, label) {
  if (!fs.existsSync(filePath)) fail(`${label} is missing: ${filePath}`);
  return fs.readFileSync(filePath, 'utf8');
}

function requireEntry(object, keyPath, label) {
  let current = object;
  for (const key of keyPath.split('.')) {
    if (current === undefined || current === null || typeof current !== 'object') {
      fail(`${label} is missing required entry "${keyPath}"`);
    }
    current = current[key];
  }
  if (current === undefined || current === null || current === '') {
    fail(`${label} is missing required entry "${keyPath}"`);
  }
  return current;
}

const contract = readJson(contractPath, 'structured-call contract');

// 1. Self-consistency of the machine contract.
const specPath = path.join(repoRoot, contract.spec ?? '');
const specText = readText(specPath, 'structured-call spec');
for (const token of ['agent_call', 'validation_failed', 'agents.calls.create']) {
  if (!specText.includes(token)) {
    fail(`spec ${contract.spec} does not reference required token "${token}"`);
  }
}

const create = requireEntry(contract, 'operations.appCreate', 'contract');
const statuses = requireEntry(contract, 'response.statuses', 'contract');
const modes = requireEntry(contract, 'request.modes', 'contract');
const formats = requireEntry(contract, 'request.outputFormats', 'contract');
for (const [name, values] of [
  ['statuses', statuses],
  ['modes', modes],
  ['outputFormats', formats],
]) {
  if (!Array.isArray(values) || values.length === 0) {
    fail(`contract request/response vocabulary "${name}" must be a non-empty array`);
  }
}
if (requireEntry(contract, 'pipeline.repairRetry.maxRepairAttempts', 'contract') !== 1) {
  fail('pipeline.repairRetry.maxRepairAttempts must be exactly 1');
}
if (requireEntry(contract, 'agentAsTool.nestingDepthMaximum', 'contract') !== 1) {
  fail('agentAsTool.nestingDepthMaximum must be exactly 1');
}

// 2. OpenAPI wire authority alignment.
const openApiPath = path.join(repoRoot, create.path ?? '', '');
const openApiFile = path.join(repoRoot, contract.wireAuthority ?? '');
const openApiText = readText(openApiFile, 'OpenAPI wire authority');
const expectedPathKey = `  ${create.path}:`;
if (!openApiText.includes(expectedPathKey)) {
  fail(`OpenAPI authority does not declare path "${create.path}"`);
}
for (const token of [
  `operationId: ${create.operationId}`,
  `x-sdkwork-permission: ${create.permission}`,
  `$ref: '#/components/schemas/${create.requestSchema}'`,
  `$ref: '#/components/schemas/${create.responseSchema}'`,
  `$ref: '#/components/schemas/${create.envelope}'`,
]) {
  if (!openApiText.includes(token)) {
    fail(`OpenAPI authority is missing "${token}" for ${create.operationId}`);
  }
}
const schemasSection = openApiText.indexOf('  schemas:');
const schemaAnchor = `    ${create.requestSchema}:`;
if (schemasSection < 0 || !openApiText.slice(schemasSection).includes(schemaAnchor)) {
  fail(`OpenAPI components.schemas does not define ${create.requestSchema}`);
}
if (!openApiText.slice(schemasSection).includes(`    ${create.responseSchema}:`)) {
  fail(`OpenAPI components.schemas does not define ${create.responseSchema}`);
}

const sdkDirPre = path.join(repoRoot, 'sdks', 'sdkwork-agents-app-sdk', 'sdkwork-agents-app-sdk-typescript', 'src');
const sdkCallsPre = readText(path.join(sdkDirPre, 'calls.ts'), 'app sdk calls module');
const sdkIndexPre = readText(path.join(sdkDirPre, 'index.ts'), 'app sdk index');

// 2b. Async lifecycle alignment (list/retrieve/202/executionMode).
const appList = requireEntry(contract, 'operations.appList', 'contract');
const appRetrieve = requireEntry(contract, 'operations.appRetrieve', 'contract');
const executionModes = requireEntry(contract, 'request.executionModes', 'contract');
for (const [name, op, extraTokens] of [
  ['appList', appList, ['AgentCallListResponse', "operationId: agents.calls.list"]],
  ['appRetrieve', appRetrieve, ["operationId: agents.calls.retrieve"]],
]) {
  const pathKey = `  ${op.path}:`;
  if (!openApiText.includes(pathKey)) {
    fail(`OpenAPI authority does not declare path "${op.path}" (${name})`);
  }
  for (const token of [`operationId: ${op.operationId}`, `x-sdkwork-permission: ${op.permission}`]) {
    if (!openApiText.includes(token)) {
      fail(`OpenAPI authority is missing "${token}" for ${op.operationId}`);
    }
  }
  for (const token of extraTokens) {
    if (!openApiText.includes(token)) {
      fail(`OpenAPI authority is missing "${token}" for ${op.operationId}`);
    }
  }
}
for (const mode of executionModes) {
  if (!openApiText.includes(`- ${mode}`)) {
    fail(`OpenAPI CreateAgentCallRequest.executionMode is missing "${mode}"`);
  }
}
if (!openApiText.includes("'202':")) {
  fail('OpenAPI agents.calls.create does not declare the 202 accepted response');
}
const appSdkTokens = ['listAgentCalls', 'getAgentCall', 'executionMode'];
for (const token of appSdkTokens) {
  if (token !== 'executionMode' && !sdkCallsPre.includes(token)) {
    fail(`app sdk calls.ts is missing "${token}"`);
  }
}
if (!sdkIndexPre.includes('listAgentCalls') || !sdkIndexPre.includes('getAgentCall')) {
  fail('app sdk index does not export the async call helpers');
}

// 2c. Usage metering alignment (summary/records feeds).
const usageSummary = requireEntry(contract, 'operations.usageSummary', 'contract');
const usageRecords = requireEntry(contract, 'operations.usageRecords', 'contract');
for (const [name, op, extraTokens] of [
  ['usageSummary', usageSummary, ['AgentUsageSummaryResponse', 'AgentUsageSummary', "operationId: agents.usage.summary.retrieve"]],
  ['usageRecords', usageRecords, ['AgentUsageRecordListResponse', 'AgentUsageRecord', "operationId: agents.usage.records.list"]],
]) {
  const pathKey = `  ${op.path}:`;
  if (!openApiText.includes(pathKey)) {
    fail(`OpenAPI authority does not declare path "${op.path}" (${name})`);
  }
  for (const token of [`operationId: ${op.operationId}`, `x-sdkwork-permission: ${op.permission}`]) {
    if (!openApiText.includes(token)) {
      fail(`OpenAPI authority is missing "${token}" for ${op.operationId}`);
    }
  }
  for (const token of extraTokens) {
    if (!openApiText.includes(token)) {
      fail(`OpenAPI authority is missing "${token}" for ${op.operationId}`);
    }
  }
}
const usageSdk = readText(path.join(sdkDirPre, 'usage.ts'), 'app sdk usage module');
for (const token of ['getUsageSummary', 'listUsageRecords', 'agents.usage']) {
  if (!usageSdk.includes(token) && token !== 'agents.usage') {
    fail(`app sdk usage.ts is missing "${token}"`);
  }
}
if (!sdkIndexPre.includes('getUsageSummary') || !sdkIndexPre.includes('listUsageRecords')) {
  fail('app sdk index does not export the usage helpers');
}

// 2d. Version governance alignment (immutable snapshots + activation).
const versionOps = [
  ['versionCreate', 'AgentVersionResponse', 'CreateAgentVersionRequest', "operationId: agents.versions.create"],
  ['versionList', 'AgentVersionListResponse', null, "operationId: agents.versions.list"],
  ['versionRetrieve', null, null, "operationId: agents.versions.retrieve"],
  ['versionActivate', null, null, "operationId: agents.versions.activate"],
];
for (const [name, schema, requestSchema, opIdToken] of versionOps) {
  const op = requireEntry(contract, `operations.${name}`, 'contract');
  if (!openApiText.includes(`  ${op.path}:`)) {
    fail(`OpenAPI authority does not declare path "${op.path}" (${name})`);
  }
  for (const token of [`operationId: ${op.operationId}`, `x-sdkwork-permission: ${op.permission}`]) {
    if (!openApiText.includes(token)) {
      fail(`OpenAPI authority is missing "${token}" for ${op.operationId}`);
    }
  }
  for (const token of [opIdToken, schema, requestSchema].filter(Boolean)) {
    if (!openApiText.includes(token)) {
      fail(`OpenAPI authority is missing "${token}" for ${op.operationId}`);
    }
  }
}
const versionSdk = readText(path.join(sdkDirPre, 'versions.ts'), 'app sdk versions module');
for (const token of ['createAgentVersion', 'listAgentVersions', 'getAgentVersion', 'activateAgentVersion']) {
  if (!versionSdk.includes(token)) {
    fail(`app sdk versions.ts is missing "${token}"`);
  }
}
if (!sdkIndexPre.includes('activateAgentVersion') || !sdkIndexPre.includes('createAgentVersion')) {
  fail('app sdk index does not export the version helpers');
}

// 2e. Webhook subscription alignment (HMAC signing + delivery ledger).
const webhookOps = [
  ['webhookCreate', 'AgentWebhookSubscriptionCreatedResponse', 'CreateWebhookSubscriptionRequest', "operationId: agents.webhooks.create"],
  ['webhookList', 'AgentWebhookSubscriptionListResponse', null, "operationId: agents.webhooks.list"],
  ['webhookRetrieve', 'AgentWebhookSubscriptionResponse', null, "operationId: agents.webhooks.retrieve"],
  ['webhookDelete', null, null, "operationId: agents.webhooks.delete"],
  ['webhookTest', 'AgentWebhookDeliveryResponse', null, "operationId: agents.webhooks.test"],
];
for (const [name, schema, requestSchema, opIdToken] of webhookOps) {
  const op = requireEntry(contract, `operations.${name}`, 'contract');
  if (!openApiText.includes(`  ${op.path}:`)) {
    fail(`OpenAPI authority does not declare path "${op.path}" (${name})`);
  }
  for (const token of [`operationId: ${op.operationId}`, `x-sdkwork-permission: ${op.permission}`]) {
    if (!openApiText.includes(token)) {
      fail(`OpenAPI authority is missing "${token}" for ${op.operationId}`);
    }
  }
  for (const token of [opIdToken, schema, requestSchema].filter(Boolean)) {
    if (!openApiText.includes(token)) {
      fail(`OpenAPI authority is missing "${token}" for ${op.operationId}`);
    }
  }
}
const webhookEventVocabulary = requireEntry(contract, 'webhooks.eventVocabulary', 'contract');
for (const eventCode of webhookEventVocabulary) {
  if (!openApiText.includes(`      - ${eventCode}`)) {
    fail(`OpenAPI AgentWebhookEventType enum is missing "${eventCode}"`);
  }
}
const webhookSdk = readText(path.join(sdkDirPre, 'webhooks.ts'), 'app sdk webhooks module');
for (const token of [
  'createWebhookSubscription',
  'listWebhookSubscriptions',
  'getWebhookSubscription',
  'deleteWebhookSubscription',
  'testWebhookDelivery',
]) {
  if (!webhookSdk.includes(token)) {
    fail(`app sdk webhooks.ts is missing "${token}"`);
  }
}
if (!sdkIndexPre.includes('createWebhookSubscription') || !sdkIndexPre.includes('testWebhookDelivery')) {
  fail('app sdk index does not export the webhook helpers');
}

// 3. Rust implementation alignment.
const domainSource = readText(
  path.join(repoRoot, 'crates', 'sdkwork-intelligence-agents-service', 'src', 'domain.rs'),
  'service domain module',
);
if (!domainSource.includes(`Self::AgentCall => "${requireEntry(contract, 'pipeline.persistence.operation', 'contract')}"`)) {
  fail('service domain AgentRuntimeExecutionOperation does not map the agent_call operation');
}
const httpSource = readText(
  path.join(repoRoot, 'crates', 'sdkwork-intelligence-agents-service', 'src', 'http.rs'),
  'service http module',
);
if (!httpSource.includes(`"${create.path}",`)) {
  fail(`service http router does not register "${create.path}"`);
}
const facadeLib = readText(
  path.join(repoRoot, 'crates', 'sdkwork-agents-runtime-facade', 'src', 'lib.rs'),
  'runtime facade lib',
);
if (!facadeLib.includes('mod structured_call')) {
  fail('runtime facade does not expose the structured_call module');
}
if (!facadeLib.includes('execute_agent_structured_call')) {
  fail('runtime facade does not export execute_agent_structured_call');
}

// 4. App SDK alignment.
const sdkDir = path.join(repoRoot, 'sdks', 'sdkwork-agents-app-sdk', 'sdkwork-agents-app-sdk-typescript', 'src');
const sdkCalls = readText(path.join(sdkDir, 'calls.ts'), 'app sdk calls module');
for (const token of ['createAgentCall', '/ai/agents/']) {
  if (!sdkCalls.includes(token)) {
    fail(`app sdk calls.ts is missing "${token}"`);
  }
}
const sdkIndex = readText(path.join(sdkDir, 'index.ts'), 'app sdk index');
if (!/from '\.\/calls(\.js|\.ts)?'/.test(sdkIndex)) {
  fail('app sdk index does not re-export the calls module');
}

console.log('Agent structured-call contract verification passed.');
