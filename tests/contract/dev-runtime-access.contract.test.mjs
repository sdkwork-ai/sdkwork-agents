import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function readEnv(relativePath) {
  return Object.fromEntries(
    read(relativePath)
      .split(/\r?\n/u)
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith('#'))
      .map((line) => {
        const separator = line.indexOf('=');
        return [line.slice(0, separator), line.slice(separator + 1)];
      }),
  );
}

function readJson(relativePath) {
  return JSON.parse(read(relativePath));
}

function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&');
}

function assertExactHttpOrigin(origin, message) {
  assert.doesNotMatch(origin, /\*/u, message);
  // Desktop shell and mini-program container origins are registered
  // non-HTTP(S) schemes; everything else must be an exact http(s) origin.
  if (/^(?:app|tauri):\/\//u.test(origin)) {
    assert.ok(origin.length > 0, message);
    return;
  }
  const parsed = new URL(origin);
  assert.ok(parsed.protocol === 'http:' || parsed.protocol === 'https:', message);
  assert.equal(parsed.origin, origin, message);
  assert.equal(parsed.search, '', message);
  assert.equal(parsed.hash, '', message);
  assert.equal(parsed.username, '', message);
  assert.equal(parsed.password, '', message);
}

test('root dev runner supports standalone gateway and remote cloud development profiles', () => {
  const runner = read('scripts/agents-dev.mjs');

  assert.match(runner, /from ['"]@sdkwork\/app-topology['"]/u);
  assert.match(runner, /topology\.defaults\.developmentProfileId/u);
  assert.match(runner, /parseArgs/u);
  assert.match(runner, /resolveSelectedProfile/u);
  assert.match(runner, /topology\.loadProfile\(selectedProfile\.profileId\)/u);
  assert.match(runner, /topology\.applyProfileEnv\(selectedProfile\.profileId\)/u);
  assert.match(runner, /topology\.listOrchestrationProcesses\(selectedProfile\.profileId\)/u);
  assert.match(runner, /topology\.listHealthSurfaces\(selectedProfile\.profileId\)/u);
  assert.match(runner, /assertLocallyRunnableProfile/u);
  assert.match(runner, /only runs development profiles/u);
  assert.match(runner, /selectedProfile\.deploymentProfile === ['"]standalone['"]/u);
  assert.match(runner, /must be an explicit remote HTTPS URL/u);
  assert.match(runner, /using deployed cloud APIs/u);
  assert.match(runner, /reportResolvedProfile/u);
  assert.match(runner, /SDKWORK_ENVIRONMENT = selectedProfile\.environment/u);
  assert.match(runner, /process\.env/u);
  assert.match(runner, /env: \{ \.\.\.runtimeEnv, \.\.\.options\.env \}/u);
  assert.match(runner, /import \{ existsSync, mkdirSync, readFileSync, rmSync, writeFileSync \} from ['"]node:fs['"]/u);
  assert.match(runner, /execFileSync\('git', \['checkout', 'HEAD', '--', \.\.\.missing\]/u);
  assert.match(runner, /gatewayProfileMatchesRuntime/u);
  assert.match(runner, /writeGatewayProfileMarker/u);
  assert.match(runner, /async function stopChild/u);
  assert.match(runner, /await Promise\.allSettled/u);
  assert.match(runner, /await shutdown\(1,/u);
  assert.doesNotMatch(runner, /killer\.unref\(\)/u);
  assert.doesNotMatch(runner, /setTimeout\(\(\) => process\.exit/u);
  assert.match(runner, /sdkwork-api-agents-standalone-gateway/u);
  assert.match(runner, /applicationIngressProcess\.crate/u);
  assert.match(runner, /pcRendererProcess\.package/u);
  assert.match(runner, /pcRendererProcess\.script/u);
  assert.match(runner, /\/healthz/u);
  assert.match(runner, /waitForGateway/u);
  assert.match(runner, /--host/u);
  assert.match(runner, /--strictPort/u);
});

test('PC development server exposes LAN access and same-origin API proxying', () => {
  const vite = read('apps/sdkwork-agents-pc/vite.config.ts');
  const runtimeConfig = read('apps/sdkwork-agents-pc/src/bootstrap/runtimeConfig.ts');
  const sdkClient = read(
    'apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src/sdk/agentsAppSdkClient.ts',
  );

  assert.match(vite, /host:\s*['"]0\.0\.0\.0['"]/u);
  assert.match(vite, /port:\s*5195/u);
  assert.match(vite, /['"]\/app\/v3\/api['"]:\s*['"]http:\/\/127\.0\.0\.1:8095['"]/u);
  assert.match(runtimeConfig, /window\.location\.origin/u);
  assert.match(sdkClient, /window\.location\.origin/u);
});

test('gateway startup output distinguishes health links from authenticated API origins', () => {
  const accessUrls = read(
    'crates/sdkwork-api-agents-standalone-gateway/src/access_urls.rs',
  );

  assert.match(accessUrls, /health URL/u);
  assert.match(accessUrls, /\/healthz/u);
  assert.match(accessUrls, /API origin \(authentication required\)/u);
});

test('operator surfaces use canonical infrastructure health endpoints', () => {
  const liveSmoke = read('scripts/agents-live-smoke.mjs');
  for (const endpoint of ['/healthz', '/livez', '/readyz', '/metrics']) {
    assert.match(
      liveSmoke,
      new RegExp(escapeRegex(endpoint), 'u'),
      `live smoke must verify ${endpoint}`,
    );
  }
  assert.doesNotMatch(liveSmoke, /fetchText\(['"]\/(?:health|ready)['"]\)/u);

  const operatorDocuments = [
    'docs/runbooks/smoke-test.md',
    'docs/runbooks/monitoring.md',
    'docs/runbooks/incident-rollback.md',
    'docs/architecture/AGENTS_LAYERING.md',
    'crates/sdkwork-intelligence-agents-service/ops/docs/operations-manual.md',
  ];
  for (const relativePath of operatorDocuments) {
    const content = read(relativePath);
    for (const endpoint of ['/healthz', '/livez', '/readyz', '/metrics']) {
      assert.match(
        content,
        new RegExp(escapeRegex(endpoint), 'u'),
        `${relativePath} must document ${endpoint}`,
      );
    }
    assert.doesNotMatch(
      content,
      /(?:GET\s+|[`'"])\/(?:health|ready)(?:\s|[`'"]|$)/mu,
      `${relativePath} must not document retired /health or /ready probes`,
    );
  }

  for (const relativePath of [
    'deployments/kubernetes/standalone-gateway-deployment.yaml',
    'crates/sdkwork-intelligence-agents-service/ops/kubernetes/deployment-hpa-ingress.yaml',
  ]) {
    const deployment = read(relativePath);
    assert.match(deployment, /path:\s+\/healthz/u);
    assert.match(deployment, /path:\s+\/readyz/u);
    assert.doesNotMatch(deployment, /path:\s+\/(?:health|ready)(?:\s|$)/mu);
  }
});

test('source topology profiles project exact CORS and IAM origins from etc', () => {
  const profiles = [
    'standalone.development',
    'standalone.test',
    'standalone.staging',
    'standalone.production',
    'standalone.demo',
    'cloud.development',
    'cloud.test',
    'cloud.staging',
    'cloud.production',
    'cloud.demo',
  ];
  const deploymentIndex = readJson('etc/sdkwork.deployment.config.json');
  const topology = readJson('specs/topology.spec.json');
  const rootManifest = readJson('sdkwork.app.config.json');
  const packageScript = read('scripts/package-cloud-gateway-config.mjs');

  assert.equal(topology.profileRoot, 'etc/topology');
  assert.deepEqual(topology.vocabulary.environment.allowed, [
    'development',
    'test',
    'staging',
    'production',
    'demo',
  ]);
  assert.equal(rootManifest.environments, undefined);
  assert.equal(rootManifest.metadata.deploymentConfig, 'etc/sdkwork.deployment.config.json');
  assert.equal(existsSync(path.join(repoRoot, 'configs')), false);
  assert.match(packageScript, /specs\/topology\.spec\.json/u);
  assert.match(packageScript, /etc\/\$\{fileName\}/u);
  assert.doesNotMatch(packageScript, /configs\//u);

  for (const profile of profiles) {
    const [deploymentProfile, environment] = profile.split('.');
    const env = readEnv(`etc/topology/${profile}.env`);

    assert.equal(deploymentIndex.profiles[profile].config, `topology/${profile}.env`);
    assert.equal(topology.profileFiles[profile], `etc/topology/${profile}.env`);
    assert.equal(env.SDKWORK_AGENTS_DEPLOYMENT_PROFILE, deploymentProfile);
    assert.equal(env.SDKWORK_AGENTS_ENVIRONMENT, environment);
    assert.equal(env.SDKWORK_ENVIRONMENT, environment);
    assert.equal(env.SDKWORK_AGENTS_PROFILE_ID, profile);
    if (deploymentProfile === 'standalone') {
      assert.equal(env.SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL, undefined);
      assert.equal(env.VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL, undefined);
    } else {
      assert.equal(
        env.VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL,
        env.SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL,
        `${profile} must project the platform IAM gateway to generic browser runtime config`,
      );
    }
    const expectedIamGateway = deploymentProfile === 'standalone'
      ? env.SDKWORK_AGENTS_APPLICATION_PUBLIC_HTTP_URL
      : env.SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL;
    assert.equal(
      env.VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL,
      expectedIamGateway,
      `${profile} must provide the PC IAM/Appbase gateway base URL`,
    );
    assert.equal(
      env.VITE_SDKWORK_AGENTS_H5_APPBASE_APP_API_BASE_URL,
      expectedIamGateway,
      `${profile} must provide the H5 IAM/Appbase gateway base URL`,
    );
    assert.equal(
      env.VITE_SDKWORK_AGENTS_H5_APPBASE_LOGIN_URL,
      expectedIamGateway,
      `${profile} must provide the H5 IAM login origin`,
    );

    if (environment === 'development') {
      // Development may declare an explicit CORS baseline (Vite dev server
      // ports plus the shared dev domains); whatever it declares must be an
      // exact-origin allowlist — wildcards never pass the CORS gate.
      const devOrigins = (env.SDKWORK_CORS_ALLOWED_ORIGINS ?? '')
        .split(',')
        .filter(Boolean);
      for (const origin of devOrigins) {
        assertExactHttpOrigin(origin, `${profile} must use an exact HTTP(S) CORS origin`);
      }
      if (deploymentProfile === 'standalone') {
        assert.equal(env.SDKWORK_AGENTS_DEV_AUTH_BYPASS, 'false');
        assert.equal(env.SDKWORK_DATABASE_ENGINE, 'postgresql');
        assert.equal(env.SDKWORK_DATABASE_SCHEMA, 'sdkwork_ai_dev');
        assert.equal(env.SDKWORK_DATABASE_SSL_MODE, 'disable');
        assert.equal(env.SDKWORK_DATABASE_URL, undefined);
        assert.equal(env.SDKWORK_DATABASE_PATH, undefined);
      } else {
        assert.equal(env.SDKWORK_DATABASE_ENGINE, undefined);
        assert.equal(env.SDKWORK_DATABASE_URL, undefined);
      }
      continue;
    }

    const origins = env.SDKWORK_CORS_ALLOWED_ORIGINS.split(',').filter(Boolean);
    assert.ok(origins.length > 0, `${profile} must declare CORS origins`);
    for (const origin of origins) {
      assertExactHttpOrigin(origin, `${profile} must use an exact HTTP(S) CORS origin`);
    }
    assert.equal(env.SDKWORK_AGENTS_DEV_AUTH_BYPASS, 'false');
    assert.equal(env.VITE_SDKWORK_AGENTS_PC_ENVIRONMENT, environment);
    assert.equal(env.VITE_SDKWORK_AGENTS_H5_ENVIRONMENT, environment);

    if (deploymentProfile === 'cloud') {
      assert.equal(
        env.SDKWORK_API_CLOUD_GATEWAY_CONFIG,
        undefined,
        `${profile} must not claim application-local cloud gateway configuration ownership`,
      );
    }
  }

  const productionOrigins = readEnv('etc/topology/cloud.production.env')
    .SDKWORK_CORS_ALLOWED_ORIGINS.split(',')
    .filter(Boolean);
  assert.ok(
    productionOrigins.includes('https://agents.sdkwork.com'),
    'cloud.production must keep the canonical web origin in its CORS allowlist',
  );
  // The fail-closed `.invalid` placeholder templates have been replaced by
  // real environment domain matrices; each still keeps the canonical web
  // origin for its environment tier.
  for (const [profile, canonical] of [
    ['standalone.test', 'https://agents-test.sdkwork.com'],
    ['standalone.staging', 'https://agents-staging.sdkwork.com'],
    ['standalone.production', 'https://agents.sdkwork.com'],
  ]) {
    const origins = readEnv(`etc/topology/${profile}.env`)
      .SDKWORK_CORS_ALLOWED_ORIGINS.split(',')
      .filter(Boolean);
    assert.ok(
      origins.includes(canonical),
      `${profile} must keep the canonical web origin ${canonical} in its CORS allowlist`,
    );
  }

  for (const manifestPath of [
    'apps/sdkwork-agents-pc/sdkwork.app.config.json',
    'apps/sdkwork-agents-h5/sdkwork.app.config.json',
    'apps/sdkwork-agents-flutter-mobile/sdkwork.app.config.json',
    'apps/sdkwork-agents-mini-program/sdkwork.app.config.json',
  ]) {
    assert.equal(readJson(manifestPath).environments, undefined, `${manifestPath} must not duplicate etc environment URLs`);
  }
});
