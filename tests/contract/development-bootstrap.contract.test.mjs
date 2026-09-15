// WORKSPACE-PATH:allow-fixture: fixtures name a foreign checkout root, drive, or home directory to exercise path handling, so the literal is the value under assertion rather than a binding this build resolves
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { mergeRepoDevBootstrapAccessTokenEnv } from '../../../sdkwork-iam/scripts/dev/create-dev-bootstrap-access-token-env.mjs';
import { ensureRustToolchain } from '../../scripts/rust-toolchain.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('development bootstrap access tokens come from the canonical IAM helper', () => {
  const environment = mergeRepoDevBootstrapAccessTokenEnv({
    repoRoot,
    env: {},
  });
  const token = environment.SDKWORK_ACCESS_TOKEN;
  const [encodedHeader, encodedPayload, signature] = token.split('.');
  const header = JSON.parse(Buffer.from(encodedHeader, 'base64url').toString('utf8'));
  const claims = JSON.parse(Buffer.from(encodedPayload, 'base64url').toString('utf8'));

  assert.equal(header.typ, 'JWT');
  assert.equal(token.split('.').length, 3);
  assert.ok(signature.length > 0);
  assert.equal(claims.app_id, 'sdkwork-agents');
  assert.equal(claims.environment, 'development');
  assert.equal(claims.token_type, 'access');
  assert.equal(claims.login_scope, 'TENANT');
  assert.equal(claims.tenant_id, '100001');
  assert.equal(claims.organization_id, '0');
});

test('root dev runner registers the application root and supplies private bootstrap credentials', () => {
  const runner = read('scripts/agents-dev.mjs');
  const databaseRunner = read('scripts/agents-database.mjs');
  const rootPackage = JSON.parse(read('package.json'));
  const pcManifest = JSON.parse(read('apps/sdkwork-agents-pc/sdkwork.app.config.json'));

  assert.match(runner, /create-dev-bootstrap-access-token-env\.mjs/);
  assert.match(runner, /mergeRepoDevBootstrapAccessTokenEnv/);
  assert.match(runner, /resolveIamDevEnv/);
  assert.match(runner, /ensurePostgresDevDatabaseReady/);
  assert.match(runner, /ensureRustToolchain/);
  assert.match(runner, /waitForGateway\(timeoutMs = 300_000\)/);
  assert.match(runner, /waitForTaskWorker\(timeoutMs = 300_000\)/);
  assert.match(runner, /application\.task-worker/);
  assert.match(runner, /cargo', \['run', '-p', taskWorkerProcess\.crate\]/);
  assert.doesNotMatch(runner, /applicationIngressProcess\.crate, '--quiet'/);
  assert.match(databaseRunner, /ensureRustToolchain/);
  assert.match(runner, /repoRoot,/);
  assert.match(runner, /SDKWORK_AGENTS_APP_ROOT = repoRoot/);
  assert.match(runner, /SDKWORK_APP_ROOT = pcAppRoot/);
  assert.match(runner, /SDKWORK_IAM_APP_ROOT/);
  assert.doesNotMatch(runner, /createHmac|randomBytes|signDevelopmentAccessToken/);
  assert.equal(rootPackage.scripts['_sdkwork:dev:standalone'], 'node scripts/agents-dev.mjs');
  assert.deepEqual(
    pcManifest.backend.accessTokenPermissionScope,
    ['iam.users.read', 'iam.organizations.read', 'iam.roles.read', 'iam.permissions.read'],
  );
});

test('standalone startup recovers the standard rustup toolchain when PATH omits Cargo', (context) => {
  const rustHome = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-agents-rust-toolchain-'));
  context.after(() => rmSync(rustHome, { force: true, recursive: true }));
  const cargoBin = path.join(rustHome, '.cargo', 'bin');
  mkdirSync(cargoBin, { recursive: true });
  writeFileSync(path.join(cargoBin, 'cargo.exe'), '', 'utf8');
  writeFileSync(path.join(cargoBin, 'rustc.exe'), '', 'utf8');

  let inspectedEnv;
  const resolvedEnv = ensureRustToolchain({
    env: {
      Path: 'C:\\Windows\\System32',
      USERPROFILE: rustHome,
    },
    platform: 'win32',
    runProcess(command, args, options) {
      assert.equal(command, 'cargo.exe');
      assert.deepEqual(args, ['--version']);
      inspectedEnv = options.env;
      return { status: 0, stderr: '', stdout: 'cargo 1.90.0' };
    },
  });

  assert.equal(resolvedEnv.Path.split(';')[0], cargoBin);
  assert.equal(inspectedEnv.Path.split(';')[0], cargoBin);
});

test('standalone gateway provisions and mounts IAM before credential entry', () => {
  const workspaceCargo = read('Cargo.toml');
  const assemblyCargo = read('crates/sdkwork-api-agents-assembly/Cargo.toml');
  const assemblyBootstrap = read('crates/sdkwork-api-agents-assembly/src/bootstrap.rs');
  const iamBootstrap = read('crates/sdkwork-api-agents-assembly/src/bootstrap/iam.rs');

  assert.match(workspaceCargo, /sdkwork-iam-embedded-application-bootstrap/);
  assert.match(workspaceCargo, /sdkwork-iam-database-host/);
  assert.match(workspaceCargo, /sdkwork-routes-iam-app-api/);
  assert.match(assemblyCargo, /sdkwork-iam-embedded-application-bootstrap\.workspace = true/);
  assert.match(assemblyCargo, /sdkwork-iam-database-host\.workspace = true/);
  assert.match(assemblyCargo, /sdkwork-routes-iam-app-api\.workspace = true/);
  assert.match(iamBootstrap, /bootstrap_iam_database_from_env\(\)/);
  assert.match(iamBootstrap, /ensure_tenant_application_from_app_root\(/);
  assert.match(iamBootstrap, /build_sdkwork_iam_app_api_router\(\)/);
  assert.match(
    assemblyBootstrap,
    /let iam_router\s*=\s*iam::wire_iam_app_router\(\)[\s\S]*let agents_router\s*=\s*sdkwork_agents_kernel_bridge::build_agents_served_router/u,
    'IAM database lifecycle must install the canonical session pool before Agents resolvers are built',
  );
  assert.match(assemblyBootstrap, /agents_router\s*\.merge\(iam_router\)/s);
  assert.match(
    assemblyBootstrap,
    /middleware::cors_layer\(\s*config\.as_ref\(\),?\s*\)/s,
  );
});

test('browser SDK token managers leave credential-entry bootstrap to the canonical IAM runtime', () => {
  for (const relativePath of [
    'apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src/session/session.ts',
    'apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-core/src/session/session.ts',
    'apps/sdkwork-agents-h5/src/components/AuthGate.tsx',
  ]) {
    const session = read(relativePath);
    assert.doesNotMatch(session, /__SDKWORK_DEVELOPMENT_ACCESS_TOKEN__/);
    assert.doesNotMatch(session, /bootstrapAccessToken/);
  }

  const iamRuntime = read('apps/sdkwork-agents-pc/src/bootstrap/iamRuntime.ts');
  assert.match(iamRuntime, /createSdkworkAppbasePcAuthRuntime/);
  assert.doesNotMatch(iamRuntime, /credentialEntry.*skipWrap/s);
});

test('Vite hands private development bootstrap tokens to IAM credential entry without a public env key', () => {
  for (const relativePath of [
    'apps/sdkwork-agents-pc/vite.config.ts',
    'apps/sdkwork-agents-h5/vite.config.ts',
  ]) {
    const vite = read(relativePath);
    assert.match(vite, /mode\s*[!=]==\s*['"]development['"]/);
    assert.match(vite, /process\.env\.SDKWORK_ACCESS_TOKEN/);
    assert.doesNotMatch(vite, /VITE_[A-Z0-9_]*ACCESS_TOKEN/);
    assert.doesNotMatch(vite, /__SDKWORK_DEVELOPMENT_ACCESS_TOKEN__/);
    assert.doesNotMatch(vite, /\.env\.development\.bootstrap\.local/);
  }
  assert.match(
    read('apps/sdkwork-agents-pc/vite.config.ts'),
    /__SDKWORK_CREDENTIAL_ENTRY_BOOTSTRAP_ACCESS_TOKEN__/,
  );
  assert.match(read('apps/sdkwork-agents-pc/vite.config.ts'), /transformIndexHtml/);
});

test('standalone PC dev uses the canonical IAM renderer bootstrap runner', () => {
  const packageManifest = JSON.parse(read('apps/sdkwork-agents-pc/package.json'));
  assert.equal(packageManifest.scripts.dev, 'pnpm dev:standalone');
  assert.match(packageManifest.scripts['dev:standalone'], /sdkwork-app dev[\s\S]*--deployment-profile standalone/u);
  assert.doesNotMatch(packageManifest.scripts['dev:standalone'], /agents-dev-env/u);
});

test('the mini-program SDK accepts a bootstrap token through its host runtime boundary', () => {
  const client = read('apps/sdkwork-agents-mini-program/packages/sdkwork-agents-mp-core/src/sdk/agentsAppSdkClient.ts');
  const bootstrap = read('apps/sdkwork-agents-mini-program/src/bootstrap/sdkClients.ts');

  assert.match(client, /configureAgentsAppSdkBootstrapAccessToken/);
  assert.match(client, /resolveAppSdkAccessToken\(currentSession\) \?\? bootstrapAccessToken/);
  assert.match(bootstrap, /configureAgentsAppSdkBootstrapAccessToken\(options\.accessToken\)/);
});

test('generated Agents app and backend SDKs retain shared TokenManager access-token propagation', () => {
  for (const relativePath of [
    'sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/generated/server-openapi/src/sdk.ts',
    'sdks/sdkwork-agents-backend-sdk/sdkwork-agents-backend-sdk-typescript/generated/server-openapi/src/sdk.ts',
  ]) {
    const sdk = read(relativePath);
    assert.match(sdk, /setTokenManager\(manager/);
    assert.match(sdk, /httpClient\.setTokenManager\(manager\)/);
  }
});
