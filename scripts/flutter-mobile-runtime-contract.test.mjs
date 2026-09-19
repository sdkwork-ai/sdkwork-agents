#!/usr/bin/env node
/**
 * Static contract test for the Agents Flutter mobile root.
 *
 * Why static: this workspace has no Dart/Flutter toolchain (`flutter` and `dart`
 * are not on PATH, and no SDK is installed), so `flutter analyze` — the
 * verification command every Flutter component spec declares — cannot run here.
 * Everything a toolchain would have caught cheaply is therefore asserted from
 * the sources instead:
 *
 *  1. Route identity parity with the mini program root (same literals, same tab
 *     order) — `APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md` section 7.
 *  2. Every `client.ai.<method>` call in the application sources resolves to a
 *     real method on the generated Dart SDK. The Dart generator flattens the
 *     sub-resource tree (`AiApi.agentsList`), so `client.ai.agents.list(...)` —
 *     the TypeScript shape — is a compile error. This check is what catches it.
 *  3. Package dependency hygiene: every `package:` import is declared in the
 *     importing package's `pubspec.yaml`. A package importing *itself* by name
 *     is exempt — pub resolves the package's own name, and that is the
 *     canonical way for a package's `test/` to reach its own `lib/` API.
 *  4. Dart directive ordering: `library;` must precede `import`.
 *  5. i18n placement: no authored `.dart` message fragments under `lib/src/i18n/`
 *     (`I18N_SPEC.md` section 6.1).
 *  6. Every `path:` dependency in every `pubspec.yaml` resolves on disk, so a
 *     mistyped relative path is caught before anyone runs `pub get`.
 */

import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(fileURLToPath(new URL(".", import.meta.url)), "..");
const flutterRoot = join(repoRoot, "apps", "sdkwork-agents-flutter-mobile");
const mpShellRoot = join(
  repoRoot,
  "apps",
  "sdkwork-agents-mini-program",
  "packages",
  "sdkwork-agents-mp-shell",
);
const sdkAiDart = join(
  repoRoot,
  "sdks",
  "sdkwork-agents-app-sdk",
  "sdkwork-agents-app-sdk-flutter",
  "generated",
  "server-openapi",
  "lib",
  "src",
  "api",
  "ai.dart",
);

const failures = [];
const checks = [];

function check(name, condition, detail) {
  checks.push({ name, ok: Boolean(condition), detail });
  if (!condition) {
    failures.push(`${name}${detail ? `\n      ${detail}` : ""}`);
  }
}

function walk(dir, filter, skipDirs = new Set()) {
  const out = [];
  if (!existsSync(dir)) {
    return out;
  }
  for (const entry of readdirSync(dir)) {
    if (
      entry === "node_modules" ||
      entry === "build" ||
      entry === ".dart_tool" ||
      skipDirs.has(entry)
    ) {
      continue;
    }
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      out.push(...walk(full, filter, skipDirs));
    } else if (filter(full)) {
      out.push(full);
    }
  }
  return out;
}

function read(file) {
  return readFileSync(file, "utf8");
}

function rel(file) {
  return relative(repoRoot, file).split(sep).join("/");
}

// ---------------------------------------------------------------------------
// 1. Route identity parity with the mini program root
// ---------------------------------------------------------------------------

const flutterRouteIdsSource = read(
  join(
    flutterRoot,
    "packages",
    "sdkwork_agents_flutter_mobile_shell",
    "lib",
    "src",
    "navigation",
    "route_ids.dart",
  ),
);
const mpRouteRegistrySource = read(join(mpShellRoot, "src", "navigation", "routeRegistry.ts"));

/** `const NAME = "value";` -> { NAME: value } from the Dart route id file. */
function parseDartConstStrings(source) {
  const map = {};
  for (const match of source.matchAll(
    /const\s+String\s+(\w+)\s*=\s*'([^']*)'\s*;/g,
  )) {
    map[match[1]] = match[2];
  }
  return map;
}

/** `export const NAME = "value" as const;` -> { NAME: value } from the TS registry. */
function parseTsConstStrings(source) {
  const map = {};
  for (const match of source.matchAll(
    /export\s+const\s+(\w+)\s*=\s*"([^"]*)"\s+as\s+const\s*;/g,
  )) {
    map[match[1]] = match[2];
  }
  return map;
}

const flutterConsts = parseDartConstStrings(flutterRouteIdsSource);
const mpConsts = parseTsConstStrings(mpRouteRegistrySource);

const routeIdPairs = [
  ["agentsConversationChatRouteId", "AGENTS_MP_CONVERSATION_CHAT_ROUTE_ID"],
  ["agentsConversationListRouteId", "AGENTS_MP_CONVERSATION_LIST_ROUTE_ID"],
  ["agentsCatalogListRouteId", "AGENTS_MP_CATALOG_LIST_ROUTE_ID"],
  ["agentsCatalogEditorRouteId", "AGENTS_MP_CATALOG_EDITOR_ROUTE_ID"],
  ["agentsLibraryListRouteId", "AGENTS_MP_LIBRARY_LIST_ROUTE_ID"],
  ["agentsAutomationIndexRouteId", "AGENTS_MP_AUTOMATION_INDEX_ROUTE_ID"],
  ["agentsProjectsListRouteId", "AGENTS_MP_PROJECTS_LIST_ROUTE_ID"],
];

for (const [flutterName, mpName] of routeIdPairs) {
  const flutterValue = flutterConsts[flutterName];
  const mpValue = mpConsts[mpName];
  check(
    `route id parity: ${flutterName} === ${mpName}`,
    Boolean(flutterValue) && flutterValue === mpValue,
    `flutter=${flutterValue ?? "<missing>"} mini-program=${mpValue ?? "<missing>"}`,
  );
}

// Tab order parity: the mini program's AGENTS_MP_TABS order must equal the
// Flutter `agentsMobileTabs` order, by resolved route id.
const mpTabsSource = read(join(mpShellRoot, "src", "navigation", "mobileTabs.ts"));
const mpTabRouteConstants = [];
for (const match of mpTabsSource.matchAll(
  /toTab\(\s*"[^"]+",\s*(AGENTS_MP_\w+),/g,
)) {
  mpTabRouteConstants.push(match[1]);
}

const flutterTabsSource = read(
  join(
    flutterRoot,
    "packages",
    "sdkwork_agents_flutter_mobile_shell",
    "lib",
    "src",
    "navigation",
    "mobile_tabs.dart",
  ),
);
const flutterTabRouteConstants = [];
for (const match of flutterTabsSource.matchAll(
  /AgentsMobileTabDescriptor\(\s*\n\s*tab:\s*AgentsMobileTabId\.\w+,\s*\n\s*routeId:\s*(\w+),/g,
)) {
  flutterTabRouteConstants.push(match[1]);
}

const mpTabRouteIds = mpTabRouteConstants.map((name) => mpConsts[name]);
const flutterTabRouteIds = flutterTabRouteConstants.map((name) => flutterConsts[name]);

check(
  "tab count is five on both mobile roots",
  mpTabRouteIds.length === 5 && flutterTabRouteIds.length === 5,
  `mini-program=${mpTabRouteIds.length} flutter=${flutterTabRouteIds.length}`,
);
check(
  "tab order and identity match the mini program root",
  JSON.stringify(mpTabRouteIds) === JSON.stringify(flutterTabRouteIds),
  `mini-program=${JSON.stringify(mpTabRouteIds)}\n      flutter=${JSON.stringify(flutterTabRouteIds)}`,
);

// ---------------------------------------------------------------------------
// 2. Every `ai.<method>` call exists on the generated Dart SDK
// ---------------------------------------------------------------------------

const aiDartSource = read(sdkAiDart);
const sdkMethodNames = new Set();
for (const match of aiDartSource.matchAll(
  /^\s{2}(?:Future|Stream)<[^\n]*?>\s+(\w+)\(/gm,
)) {
  sdkMethodNames.add(match[1]);
}

check(
  "generated Dart SDK exposes a method surface",
  sdkMethodNames.size > 50,
  `parsed ${sdkMethodNames.size} methods from ${rel(sdkAiDart)}`,
);

const dartSources = walk(flutterRoot, (file) => file.endsWith(".dart"));
const badAiCalls = [];
for (const file of dartSources) {
  const source = read(file);
  for (const match of source.matchAll(/\bai\.([A-Za-z_]\w*)/g)) {
    // Only a `<expr>ai.` access on an SDK client is a call site. Permission
    // identifiers such as `ai.agents.read` and doc comments mention `ai.`
    // without touching the client, so require the `client.` prefix.
    const prefix = source.slice(Math.max(0, match.index - "client.".length), match.index);
    if (prefix !== "client.") {
      continue;
    }
    const name = match[1];
    if (!sdkMethodNames.has(name)) {
      const line = source.slice(0, match.index).split(/\r?\n/).length;
      badAiCalls.push(`${rel(file)}:${line} -> client.ai.${name}`);
    }
  }
}

check(
  "every ai.<method> call exists on the generated Dart SDK",
  badAiCalls.length === 0,
  badAiCalls.length === 0
    ? undefined
    : `The Dart generator flattens the sub-resource tree, so an ` +
      `AiApi.agents.list-shaped call is a compile error:\n      ` +
      badAiCalls.join("\n      "),
);

// ---------------------------------------------------------------------------
// 3 + 4 + 5. Package hygiene, directive ordering, i18n placement
// ---------------------------------------------------------------------------

const packageDirs = walk(join(flutterRoot, "packages"), (file) =>
  file.endsWith(`${sep}pubspec.yaml`),
);

function parsePubspecNames(source) {
  const names = new Set();
  let section = null;
  for (const rawLine of source.split(/\r?\n/)) {
    if (/^\S/.test(rawLine)) {
      const header = rawLine.match(/^(\w+):/);
      section = header ? header[1] : null;
      continue;
    }
    if (section !== "dependencies" && section !== "dev_dependencies") {
      continue;
    }
    const dep = rawLine.match(/^\s{2}([A-Za-z_][\w-]*):/);
    if (dep) {
      names.add(dep[1]);
    }
  }
  return names;
}

/** The package's own name, which pub resolves for `package:` self-imports. */
function parsePubspecPackageName(source) {
  const match = source.match(/^name:\s*([A-Za-z_][\w-]*)\s*$/m);
  return match ? match[1] : null;
}

/** Every package root that contributes Dart sources, including the app root. */
const dartPackageRoots = [
  // The app root's public exports live in `lib/`; `packages/**` is walked
  // separately with its own manifest, so skip it here to avoid attributing a
  // package file to the app manifest.
  {
    root: flutterRoot,
    pubspec: join(flutterRoot, "pubspec.yaml"),
    skipDirs: new Set(["packages"]),
  },
  ...packageDirs.map((pubspec) => ({
    root: join(pubspec, ".."),
    pubspec,
    skipDirs: new Set(),
  })),
];

const undeclaredImports = [];
const directiveOrderViolations = [];
const i18nFragmentViolations = [];

for (const { root, pubspec, skipDirs } of dartPackageRoots) {
  const pubspecSource = read(pubspec);
  const declared = parsePubspecNames(pubspecSource);
  const selfName = parsePubspecPackageName(pubspecSource);
  const files = walk(root, (file) => file.endsWith(".dart"), skipDirs);
  for (const file of files) {
    const source = read(file);
    const relPath = rel(file);

    for (const match of source.matchAll(/^import\s+'package:([^/]+)\//gm)) {
      const name = match[1];
      if (name === selfName) {
        continue;
      }
      if (!declared.has(name)) {
        undeclaredImports.push(`${relPath} imports package:${name} (not in ${rel(pubspec)})`);
      }
    }

    const lines = source.split(/\r?\n/);
    const libraryIndex = lines.findIndex((line) => /^library\b/.test(line));
    const importIndex = lines.findIndex((line) => /^import\s/.test(line));
    if (libraryIndex >= 0 && importIndex >= 0 && libraryIndex > importIndex) {
      directiveOrderViolations.push(
        `${relPath}: library declaration at line ${libraryIndex + 1} follows an import at line ${importIndex + 1}`,
      );
    }

    if (relPath.includes("/lib/src/i18n/") && relPath.endsWith(".dart")) {
      i18nFragmentViolations.push(relPath);
    }
  }
}

check(
  "every package: import is declared in the importing package's pubspec",
  undeclaredImports.length === 0,
  undeclaredImports.join("\n      "),
);
check(
  "library declarations precede imports",
  directiveOrderViolations.length === 0,
  directiveOrderViolations.join("\n      "),
);
check(
  "no authored .dart message fragments under lib/src/i18n/",
  i18nFragmentViolations.length === 0,
  i18nFragmentViolations.join("\n      "),
);

// ---------------------------------------------------------------------------
// 3b. Every `path:` dependency resolves on disk
// ---------------------------------------------------------------------------

const brokenPathDeps = [];
for (const { pubspec } of dartPackageRoots) {
  const dir = join(pubspec, "..");
  const source = read(pubspec);
  for (const match of source.matchAll(/^\s+path:\s*(\S+)\s*$/gm)) {
    const declaredPath = match[1];
    if (!existsSync(join(dir, declaredPath))) {
      const line = source.slice(0, match.index).split(/\r?\n/).length;
      brokenPathDeps.push(
        `${rel(pubspec)}:${line} -> ${declaredPath} does not resolve from ${rel(dir)}`,
      );
    }
  }
}
check(
  "every path: dependency resolves on disk",
  brokenPathDeps.length === 0,
  brokenPathDeps.join("\n      "),
);

// ---------------------------------------------------------------------------
// Composition: the root declares every capability package, and the tab hosts
// mount exactly one host per shell tab.
// ---------------------------------------------------------------------------

const rootPubspec = read(join(flutterRoot, "pubspec.yaml"));
const expectedRootDeps = [
  "sdkwork_agents_flutter_mobile_core",
  "sdkwork_agents_flutter_mobile_commons",
  "sdkwork_agents_flutter_mobile_shell",
  "sdkwork_agents_flutter_mobile_agents",
  "sdkwork_agents_flutter_mobile_conversation",
  "sdkwork_agents_flutter_mobile_library",
  "sdkwork_agents_flutter_mobile_projects",
  "sdkwork_agents_flutter_mobile_automation",
];
const missingRootDeps = expectedRootDeps.filter(
  (name) => !new RegExp(`^\\s{2}${name}:`, "m").test(rootPubspec),
);
check(
  "root pubspec declares every client package",
  missingRootDeps.length === 0,
  missingRootDeps.join(", "),
);

const shellSource = read(join(flutterRoot, "lib", "app", "tab_hosts.dart"));
const hostCount = [...shellSource.matchAll(/return _ControllerTabHost</g)].length;
check(
  "tab hosts cover all five tabs",
  hostCount === 5,
  `found ${hostCount} host widgets`,
);

// ---------------------------------------------------------------------------
// Diagnostic (not a gate): artifacts whose generator is absent here.
//
// `pubspec.lock` is produced by `pub`, and neither `flutter` nor `dart` is on
// PATH in this workspace, so a regenerated lock cannot be produced. The
// committed lock is already stale with respect to the root manifest — it
// carries entries for only a subset of the declared path packages — and that
// staleness predates the mobile shell work. It is reported rather than
// asserted on purpose: a hand-edited lock would fail
// `pub get --enforce-lockfile` with an opaque checksum mismatch, whereas a
// plainly stale lock fails with "lockfile is out of date" and names its own
// remedy (`flutter pub get`).
// ---------------------------------------------------------------------------

const rootLockPath = join(flutterRoot, "pubspec.lock");
const staleLockPackages = [];
if (existsSync(rootLockPath)) {
  const lockSource = read(rootLockPath);
  for (const match of rootPubspec.matchAll(/^\s+path:\s*(\S+)\s*$/gm)) {
    const targetPubspec = join(flutterRoot, match[1], "pubspec.yaml");
    if (!existsSync(targetPubspec)) {
      continue;
    }
    const name = parsePubspecPackageName(read(targetPubspec));
    if (name && !new RegExp(`^  ${name}:`, "m").test(lockSource)) {
      staleLockPackages.push(name);
    }
  }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

for (const entry of checks) {
  console.log(`${entry.ok ? "PASS" : "FAIL"}  ${entry.name}`);
}

if (staleLockPackages.length > 0) {
  console.log(
    `\nNOTE  pubspec.lock is stale: ${staleLockPackages.length} declared path ` +
      `package(s) have no lock entry.\n` +
      `      ${staleLockPackages.join(", ")}\n` +
      `      Remedy: run \`flutter pub get\` in apps/sdkwork-agents-flutter-mobile on a ` +
      `machine with the Flutter SDK and commit the regenerated lock.\n` +
      `      Not asserted: this workspace has no Dart toolchain, and a hand-written ` +
      `lock would fail \`--enforce-lockfile\` with a checksum mismatch instead of ` +
      `the clear "lockfile is out of date".`,
  );
}

if (failures.length > 0) {
  console.error(`\n${failures.length} Flutter mobile contract check(s) failed:\n`);
  for (const failure of failures) {
    console.error(`  - ${failure}`);
  }
  process.exit(1);
}

console.log(`\nFlutter mobile runtime contract: ${checks.length} checks passed.`);
