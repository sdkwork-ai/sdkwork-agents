import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const workflow = JSON.parse(fs.readFileSync(path.join(root, 'agent.workflow.json'), 'utf8'));

test('release workflow keeps install fail-closed and covers declared server deployment targets', () => {
  assert.equal(workflow.lifecycle.install.length, 1);
  assert.equal(workflow.lifecycle.install[0].run, 'pnpm install --frozen-lockfile');
  assert.ok(Array.isArray(workflow.lifecycle.sign) && workflow.lifecycle.sign.length > 0);
  assert.ok(Array.isArray(workflow.lifecycle.sbom) && workflow.lifecycle.sbom.length > 0);
  assert.deepEqual(
    new Set(workflow.targets.map((target) => target.id)),
    new Set([
      'linux-x64-standalone-server-tar-gz',
      'container-x64-cloud-container-kubernetes-tar-gz',
    ]),
  );
});

test('supply-chain evidence refuses to run without selected artifact evidence', () => {
  const env = { ...process.env };
  for (const name of [
    'SDKWORK_PACKAGE_ID',
    'SDKWORK_PACKAGE_ARTIFACT_PATH',
    'SDKWORK_PACKAGE_VERSION',
    'SDKWORK_RELEASE_SIGNING_PRIVATE_KEY',
    'SDKWORK_RELEASE_SIGNING_KEY_FILE',
  ]) {
    delete env[name];
  }
  const result = spawnSync(
    process.execPath,
    ['scripts/release/workflow-supply-chain-evidence.mjs', 'sign'],
    { cwd: root, env, encoding: 'utf8' },
  );
  assert.notEqual(result.status, 0);
  assert.match(`${result.stdout}${result.stderr}`, /SDKWORK_PACKAGE_ID is required/u);
});
