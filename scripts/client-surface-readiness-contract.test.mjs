import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function mustExist(relativePath) {
  const absolute = path.join(root, relativePath);
  assert.ok(fs.existsSync(absolute), `${relativePath} must exist`);
  return fs.readFileSync(absolute, "utf8");
}

function mustExport(relativePath, exportName) {
  const content = mustExist(relativePath);
  assert.match(
    content,
    new RegExp(`export\\s+(function|const|class|type)\\s+${exportName}\\b|export\\s*\\{[^}]*\\b${exportName}\\b`),
    `${relativePath} must export ${exportName}`,
  );
}

function listSourceFiles(relativePath) {
  const absolute = path.join(root, relativePath);
  const entries = fs.readdirSync(absolute, { withFileTypes: true });
  return entries.flatMap((entry) => {
    const childRelativePath = path.join(relativePath, entry.name);
    if (entry.isDirectory()) {
      return listSourceFiles(childRelativePath);
    }
    return /\.(ts|tsx|js|jsx)$/u.test(entry.name) ? [childRelativePath] : [];
  });
}

function assertNoLaunchDebtCopy(label, relativePath) {
  const forbiddenCopyPatterns = [
    /后续版本/u,
    /后续开放/u,
    /未来版本/u,
    /敬请期待/u,
    /待开放/u,
    /not implemented/iu,
    /coming soon/iu,
    /under construction/iu,
  ];
  const matches = listSourceFiles(relativePath).flatMap((filePath) => {
    const content = mustExist(filePath);
    return forbiddenCopyPatterns
      .filter((pattern) => pattern.test(content))
      .map((pattern) => `${filePath} matches ${pattern}`);
  });
  assert.deepEqual(matches, [], `${label} source must not expose launch-debt copy`);
}

const pcAgents = "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/src";
const h5Agents = "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents/src";
const pcApp = "apps/sdkwork-agents-pc/src";
const h5App = "apps/sdkwork-agents-h5/src";
const pcCoreSdk = "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src/sdk";
const h5CoreSdk = "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-core/src/sdk";

mustExport(`${pcApp}/AuthGate.tsx`, "AuthGate");
mustExport(`${h5App}/components/AuthGate.tsx`, "AuthGate");
mustExport(`${pcAgents}/pages/AgentChatView.tsx`, "AgentChatView");
mustExport(`${h5Agents}/pages/AgentChatView.tsx`, "AgentChatView");
mustExport(`${pcAgents}/services/AgentChatService.ts`, "AgentChatService");
mustExport(`${h5Agents}/services/AgentChatService.ts`, "AgentChatService");
mustExport(`${pcCoreSdk}/runtimeEnv.ts`, "readRuntimeEnv");
mustExport(`${h5CoreSdk}/runtimeEnv.ts`, "readRuntimeEnv");
assertNoLaunchDebtCopy("PC agents", pcAgents);
assertNoLaunchDebtCopy("H5 agents", h5Agents);

const h5AgentsPkg = JSON.parse(
  mustExist("apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents/package.json"),
);
assert.equal(
  h5AgentsPkg.dependencies?.["@sdkwork/agents-app-sdk"],
  undefined,
  "H5 agents capability package must not depend on @sdkwork/agents-app-sdk directly",
);
assert.equal(
  h5AgentsPkg.dependencies?.["@sdkwork/utils"],
  "workspace:*",
  "H5 agents capability package must keep @sdkwork/utils for approved client business ID helpers",
);

assert.match(
  mustExist(`${h5Agents}/services/businessIdentifiers.ts`),
  /const\s+CLIENT_SURFACE\s*=\s*"h5"/u,
  "H5 business identifiers must use the h5 client surface prefix after materialization",
);

const h5Materializer = mustExist("scripts/materialize-h5-agents-from-pc.mjs");
assert.doesNotMatch(
  h5Materializer,
  /rmSync\(\s*h5AgentsRoot\s*,\s*\{\s*recursive:\s*true/u,
  "H5 agents materializer must not recursively remove the package root because that deletes package-local workspace dependency links",
);
assert.match(
  h5Materializer,
  /entry\.name\s*===\s*"node_modules"/u,
  "H5 agents materializer must explicitly skip package-local node_modules while syncing source",
);

const clientAppSurfaceMaterializer = mustExist("scripts/materialize-client-app-surfaces.mjs");
assert.doesNotMatch(
  clientAppSurfaceMaterializer,
  /sdkwork-agents-app-sdk-typescript[\\/]+generated[\\/]+server-openapi/u,
  "Client app surface materializer must not generate aliases or workspace entries pointing at SDK generated transport output",
);
assert.match(
  clientAppSurfaceMaterializer,
  /sdks\/sdkwork-agents-app-sdk\/sdkwork-agents-app-sdk-typescript\/src\/index\.ts/u,
  "Client app surface materializer must generate aliases pointing at the composed app SDK facade",
);
assert.match(
  clientAppSurfaceMaterializer,
  /- "sdks\/sdkwork-agents-app-sdk\/sdkwork-agents-app-sdk-typescript"/u,
  "Client app surface materializer must register the composed app SDK TypeScript workspace root",
);

const mpAgentsPage = mustExist("apps/sdkwork-agents-mini-program/src/pages/agents/index.js");
const mpAgentsPageWxml = mustExist("apps/sdkwork-agents-mini-program/src/pages/agents/index.wxml");
assert.doesNotMatch(
  mpAgentsPageWxml,
  /<web-view/u,
  "agents index page must be native (WebView moved to agents-h5)",
);
// The page loads agents through the mini program runtime bundle, which resolves
// the generated SDK client itself (`getAgentsMpRuntimeServices` ->
// `getAgentsAppSdkClient`). The page consumes the capability services and must
// not reach for the SDK client directly.
assert.match(
  mpAgentsPage,
  /getAgentsMpRuntimeServices/u,
  "agents index must load agents via runtime SDK services",
);
assert.doesNotMatch(
  mpAgentsPage,
  /getAgentsMpSdkClient/u,
  "agents index must consume runtime services instead of resolving the SDK client itself",
);
// The editor bridge target lives in the page script; the template only wires the
// handler, so the bridge page path is asserted where it is authored.
assert.match(
  mpAgentsPage,
  /\/pages\/agents-h5\/index/u,
  "agents index must link to the explicit editor bridge page",
);
assert.match(
  mpAgentsPageWxml,
  /onOpenFullVersion/u,
  "agents index wxml must wire the editor bridge action",
);

const pcAppSource = mustExist(`${pcApp}/App.tsx`);
assert.match(pcAppSource, /AuthGate/u, "PC App.tsx must wrap the workbench with AuthGate");
assert.match(pcAppSource, /AgentsWorkbench/u, "PC App.tsx must wire the production workbench");
assert.match(
  mustExist(`${pcApp}/components/WorkbenchLayout.tsx`),
  /AgentWorkspace/u,
  "PC workbench must wire the SDK-backed agent workspace",
);

const h5AppSource = mustExist(`${h5App}/App.tsx`);
assert.match(h5AppSource, /AuthGate/u, "H5 App.tsx must wrap routes with AuthGate");
assert.match(h5AppSource, /AgentChatView/u, "H5 App.tsx must wire AgentChatView");
assert.match(h5AppSource, /CHAT_ROUTE/u, "H5 App.tsx must wire the chat route constant");

console.log("client surface readiness contract passed.");
