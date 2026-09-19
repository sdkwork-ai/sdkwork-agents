import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const bundlePath = path.join(
  root,
  "apps/sdkwork-agents-mini-program/src/runtime/agents-app.js",
);
const runtimeEnvPath = path.join(
  root,
  "apps/sdkwork-agents-mini-program/src/runtime/runtime-env.js",
);
const buildManifestPath = path.join(
  root,
  "apps/sdkwork-agents-mini-program/src/runtime/build-manifest.json",
);

assert.ok(fs.existsSync(bundlePath), "mini-program runtime bundle must exist; run pnpm --filter @sdkwork/agents-mini-program build");
assert.ok(fs.existsSync(runtimeEnvPath), "selected mini-program runtime env must exist");
assert.ok(fs.existsSync(buildManifestPath), "mini-program build manifest must exist");

const bundle = fs.readFileSync(bundlePath, "utf8");
assert.ok(bundle.length > 10_000, "runtime bundle looks truncated");

for (const marker of [
  "bootstrapAgentsMiniProgram",
  "getAgentsMpSdkClient",
  "createAgentsAppSdkClientConfig",
]) {
  assert.match(bundle, new RegExp(marker), `runtime bundle must export ${marker}`);
}

const runtimeEnv = fs.readFileSync(runtimeEnvPath, "utf8");
const buildManifest = JSON.parse(fs.readFileSync(buildManifestPath, "utf8"));
assert.match(runtimeEnv, /SDKWORK_PROFILE_ID/u);
assert.equal(buildManifest.profileId, `${buildManifest.deploymentProfile}.${buildManifest.environment}`);
assert.equal(buildManifest.runtimeTarget, "mini-program");
assert.equal(buildManifest.platform, "MP_WEIXIN");

const appSource = fs.readFileSync(
  path.join(root, "apps/sdkwork-agents-mini-program/src/app.js"),
  "utf8",
);
assert.match(appSource, /require\("\.\/runtime\/runtime-env"\)/u);
assert.doesNotMatch(appSource, /agentsAppApiBaseUrl:\s*"http:\/\/127\.0\.0\.1/u);

for (const forbiddenMarker of [
  "generated/server-openapi",
  "domain-transport-sdk",
  "domain-transport-typescript",
]) {
  assert.doesNotMatch(
    bundle,
    new RegExp(forbiddenMarker.replaceAll("/", "[\\\\/]"), "u"),
    `runtime bundle must not expose generated SDK transport source path ${forbiddenMarker}`,
  );
}

// ---------------------------------------------------------------------------
// Route projection: `app.json` must be the projection of the route
// contributions the capability packages publish
// (`MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md` sections 5 and 13).
// ---------------------------------------------------------------------------

/** Pages that exist for platform reasons and are not capability routes. */
const PLATFORM_ONLY_PAGES = new Set([
  // First-run / AuthGate redirect target.
  "pages/home/index",
  // Native <web-view> bridge the experts tab can open.
  "pages/agents-h5/index",
]);

/**
 * Evaluates the CJS bundle.
 *
 * The bundle is CommonJS for the mini program loader, but the application root
 * declares `"type": "module"`, so `require()` would treat it as ESM. Compiling it
 * with `new Function` reuses this realm's globals and needs no `.cjs` copy.
 */
function loadRuntimeBundle() {
  const moduleShim = { exports: {} };
  const factory = new Function("module", "exports", bundle);
  factory(moduleShim, moduleShim.exports);
  return moduleShim.exports;
}

const runtime = loadRuntimeBundle();
const routes = runtime.createRoutes();
assert.ok(Array.isArray(routes) && routes.length > 0, "runtime must assemble route contributions");

const appJson = JSON.parse(
  fs.readFileSync(path.join(root, "apps/sdkwork-agents-mini-program/src/app.json"), "utf8"),
);
const declaredPages = new Set(appJson.pages);

for (const pagePath of runtime.listRootPages(routes)) {
  assert.ok(
    declaredPages.has(pagePath),
    `app.json pages must declare the projected root page ${pagePath}`,
  );
}

assert.deepEqual(
  [...runtime.listSubpackages(routes)],
  appJson.subPackages.map((entry) => ({ root: entry.root, pages: [...entry.pages] })),
  "app.json subPackages must be the projection of the subpackage route contributions",
);

// Every non-platform page in `app.json` must be owned by a route contribution,
// so a page cannot be added without declaring its route identity.
const projectedPages = new Set(runtime.listRootPages(routes));
for (const entry of appJson.subPackages) {
  for (const pagePath of entry.pages) {
    projectedPages.add(`${entry.root}/${pagePath}`);
  }
}
for (const pagePath of appJson.pages) {
  if (PLATFORM_ONLY_PAGES.has(pagePath)) {
    continue;
  }
  assert.ok(
    projectedPages.has(pagePath),
    `${pagePath} is not projected from any route contribution`,
  );
}

// The tab bar must be exactly the shared mobile tab set, so the mini program,
// H5, and Flutter roots cannot drift apart.
assert.equal(appJson.tabBar?.custom, true, "the tab bar must be custom");
assert.deepEqual(
  appJson.tabBar.list.map((entry) => entry.pagePath),
  runtime.AGENTS_MP_TABS.map((entry) => entry.pagePath),
  "tabBar list must match AGENTS_MP_TABS",
);
for (const entry of appJson.tabBar.list) {
  const descriptor = runtime.resolveAgentsMpTabByRouteId(
    runtime.AGENTS_MP_TABS.find((tab) => tab.pagePath === entry.pagePath)?.routeId,
  );
  assert.ok(descriptor, `tabBar page ${entry.pagePath} must resolve to a tab descriptor`);
  assert.ok(declaredPages.has(entry.pagePath), `tabBar page ${entry.pagePath} must be in app.json pages`);
}

// Every custom tab bar page must render the shared tab bar component.
for (const entry of appJson.tabBar.list) {
  const pageDir = path.join(root, "apps/sdkwork-agents-mini-program/src", entry.pagePath);
  assert.ok(
    fs.existsSync(`${pageDir}.wxml`) && fs.existsSync(`${pageDir}.js`),
    `tab page ${entry.pagePath} must define .js and .wxml files`,
  );
  const pageSource = fs.readFileSync(`${pageDir}.js`, "utf8");
  assert.match(
    pageSource,
    /getTabBar/u,
    `tab page ${entry.pagePath} must sync the custom tab bar state`,
  );
}

console.log("mini-program runtime contract passed.");
