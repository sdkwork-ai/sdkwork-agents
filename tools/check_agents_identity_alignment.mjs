#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  collectLegacyProviderSessionIdentity,
} from '../../sdkwork-specs/tools/lib/provider-session-identity.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];

const EXPECTED = {
  tenantId: '100001',
  organizationId: '0',
  appId: 'sdkwork-agents',
};

const CLIENT_MANIFEST_KEYS = {
  'apps/sdkwork-agents-pc/agent.app.config.json': 'agents-pc',
  'apps/sdkwork-agents-h5/agent.app.config.json': 'agents-h5',
  'apps/sdkwork-agents-mini-program/agent.app.config.json': 'agents-mini-program',
  'apps/sdkwork-agents-flutter-mobile/agent.app.config.json': 'agents-flutter-mobile',
};

const FORBIDDEN_PATTERNS = [
  { label: 'legacy tenant_id: 1 fixture', pattern: /\btenant_id:\s*1\b/g },
  { label: 'legacy tenant_id: 7 fixture', pattern: /\btenant_id:\s*7\b/g },
  { label: 'legacy organization_id: 10 fixture', pattern: /\borganization_id:\s*10\b/g },
  { label: 'legacy organization_id: 70 fixture', pattern: /\borganization_id:\s*70\b/g },
  { label: 'legacy organization_id string "10"', pattern: /organization_id:\s*"10"/g },
  { label: 'legacy organizationId string "10"', pattern: /"organizationId":\s*"10"/g },
  { label: 'legacy with_organization_id("10")', pattern: /with_organization_id\("10"\)/g },
  { label: 'doubled sdkwork-agents app.key', pattern: /"key":\s*"sdkwork-agents-sdkwork-agents-/g },
  { label: 'legacy env tenant id =1001', pattern: /TENANT_ID=1001(?:\r?\n|$)/g },
  { label: 'legacy env tenant id =1', pattern: /TENANT_ID=1(?:\r?\n|$)/g },
  { label: 'legacy AgentListQuery::for_tenant(1)', pattern: /for_tenant\(1\)/g },
  { label: 'legacy PolicySubject tenant.1', pattern: /PolicySubject::new\([^,]+,\s*"tenant\.1"\)/g },
  { label: 'legacy fixture uuid tenant segment _7_', pattern: /agent_[a-z_]+_7_/g },
  { label: 'surface-specific backend.appId on client manifest', pattern: /"appId":\s*"sdkwork-agents-(?:pc|h5|mini-program|flutter-mobile)"/g },
];

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, relativePath), 'utf8').replace(/^\uFEFF/u, ''));
}

function assert(condition, message) {
  if (!condition) failures.push(message);
}

function walk(dir, files = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'node_modules' || entry.name === 'dist' || entry.name === 'target' || entry.name === 'generated') {
        continue;
      }
      walk(absolute, files);
      continue;
    }
    if (/\.(rs|ts|tsx|mjs|json|sql|env\.example)$/u.test(entry.name)) {
      files.push(absolute);
    }
  }
  return files;
}

function checkManifest(relativePath, expectedKey = null) {
  const manifest = readJson(relativePath);
  assert(manifest.backend?.tenantId === EXPECTED.tenantId, `${relativePath}: backend.tenantId must be ${EXPECTED.tenantId}`);
  assert(manifest.backend?.organizationId === EXPECTED.organizationId, `${relativePath}: backend.organizationId must be ${EXPECTED.organizationId}`);
  if (manifest.backend?.appId != null) {
    assert(manifest.backend.appId === EXPECTED.appId, `${relativePath}: backend.appId must be ${EXPECTED.appId}`);
  }
  if (expectedKey) {
    assert(manifest.app?.key === expectedKey, `${relativePath}: app.key must be ${expectedKey}`);
  }
}

checkManifest('agent.app.config.json');
for (const [relativePath, expectedKey] of Object.entries(CLIENT_MANIFEST_KEYS)) {
  checkManifest(relativePath, expectedKey);
}

const envExample = fs.readFileSync(path.join(repoRoot, '.env.example'), 'utf8');
assert(envExample.includes('SDKWORK_TENANT_ID=100001'), '.env.example must declare SDKWORK_TENANT_ID=100001');
assert(envExample.includes('SDKWORK_ORGANIZATION_ID=0'), '.env.example must declare SDKWORK_ORGANIZATION_ID=0');

for (const file of walk(repoRoot)) {
  const relative = path.relative(repoRoot, file).replaceAll('\\', '/');
  if (relative.startsWith('sdks/') && relative.includes('/generated/')) continue;
  if (relative.includes('check_agents_identity_alignment.mjs')) continue;
  const text = fs.readFileSync(file, 'utf8');
  for (const { label, pattern } of FORBIDDEN_PATTERNS) {
    pattern.lastIndex = 0;
    if (pattern.test(text)) {
      failures.push(`${relative}: forbidden ${label}`);
    }
  }
}

for (const violation of collectLegacyProviderSessionIdentity(repoRoot)) {
  const relative = path.relative(repoRoot, violation.filePath).replaceAll('\\', '/');
  const location = violation.line > 0 ? `:${violation.line}` : '';
  failures.push(`${relative}${location}: retired provider Session identity ${violation.legacy}`);
}

if (failures.length > 0) {
  console.error('agents identity alignment failures:');
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log('agents identity alignment passed');
