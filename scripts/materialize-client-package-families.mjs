#!/usr/bin/env node

// Materializes the missing package-family roles for the SDKWork Agents
// mini-program and Flutter mobile client roots:
//
//   apps/sdkwork-agents-mini-program/packages/sdkwork-agents-mp-{commons,shell,agents}
//   apps/sdkwork-agents-flutter-mobile/packages/sdkwork_agents_flutter_mobile_{commons,shell,agents}
//
// Authority:
//   - MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md
//   - FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md
//   - APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md (package taxonomy / dependency direction)
//
// Design rules:
// - Idempotent: existing files are never overwritten.
// - Scope: only adds packages; never rewrites existing packages or root files.
// - Capability packages never construct SDK clients; they receive injected
//   clients/ports (ALIGNMENT_SPEC section 5).

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const applicationCode = 'agents';

const created = [];
const skipped = [];

function writeFile(absolutePath, content) {
  fs.mkdirSync(path.dirname(absolutePath), { recursive: true });
  if (fs.existsSync(absolutePath)) {
    skipped.push(path.relative(repoRoot, absolutePath).replaceAll('\\', '/'));
    return false;
  }
  fs.writeFileSync(absolutePath, content);
  created.push(path.relative(repoRoot, absolutePath).replaceAll('\\', '/'));
  return true;
}

function writeJson(absolutePath, value) {
  return writeFile(absolutePath, `${JSON.stringify(value, null, 2)}\n`);
}

function specsRefs(archSpecFile, extra = []) {
  return [
    {
      file: archSpecFile,
      path: '../../../../../sdkwork-specs/' + archSpecFile,
      purpose: 'Client application root architecture.',
    },
    {
      file: 'APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md',
      path: '../../../../../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md',
      purpose: 'Cross-client package taxonomy and dependency direction.',
    },
  ].concat(extra);
}

const alignmentSpec = {
  file: 'APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md',
  path: '../../../../../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md',
  purpose: 'Cross-client package taxonomy and dependency direction.',
};
const frontendSpec = {
  file: 'FRONTEND_SPEC.md',
  path: '../../../../../sdkwork-specs/FRONTEND_SPEC.md',
  purpose: 'Frontend package layering and SDK injection boundaries.',
};
const i18nSpec = {
  file: 'I18N_SPEC.md',
  path: '../../../../../sdkwork-specs/I18N_SPEC.md',
  purpose: 'Runtime locale configuration and message catalog rules.',
};
const paginationSpec = {
  file: 'PAGINATION_SPEC.md',
  path: '../../../../../sdkwork-specs/PAGINATION_SPEC.md',
  purpose: 'Interactive list pagination.',
};

// ===========================================================================
// Mini program
// ===========================================================================

const mpRoot = path.join(repoRoot, 'apps', 'sdkwork-agents-mini-program');
const mpArchSpec = 'MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md';
const mpUiSpec = {
  file: 'APP_MINI_PROGRAM_UI_SPEC.md',
  path: '../../../../../sdkwork-specs/APP_MINI_PROGRAM_UI_SPEC.md',
  purpose: 'Mini program UI package rules.',
};

function mpComponentSpec({ packageDirName, capability, layerRole, extraContracts = {} }) {
  return {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: packageDirName,
      displayName: packageDirName,
      version: '0.1.0',
      type: 'typescript-package',
      root: `apps/sdkwork-agents-mini-program/packages/${packageDirName}`,
      domain: applicationCode,
      capability,
      surface: 'app',
      languages: ['typescript'],
      generated: false,
      manifests: ['package.json', 'specs/component.spec.json'],
    },
    canonicalSpecs: [{
      file: mpArchSpec,
      path: '../../../../../sdkwork-specs/' + mpArchSpec,
      purpose: 'Mini program application root architecture.',
    }, alignmentSpec].concat(
      layerRole === 'frontend-commons' ? [frontendSpec, mpUiSpec, i18nSpec]
        : layerRole === 'frontend-shell' ? [frontendSpec, mpUiSpec]
          : [mpUiSpec, paginationSpec, i18nSpec],
    ),
    contracts: {
      layerRole,
      publicExports: ['.'],
      sdkClients: [],
      sdkDependencies: [],
      dependencyApiExports: [],
      dependencyApiSurfaces: [],
      ...extraContracts,
    },
    verification: {
      commands: ['pnpm --filter @sdkwork/agents-mini-program typecheck'],
    },
  };
}

function mpPackageJson(packageDirName, npmName, dependencies) {
  return {
    name: npmName,
    private: true,
    version: '0.1.0',
    type: 'module',
    exports: {
      '.': {
        types: './src/index.ts',
        import: './src/index.ts',
        default: './src/index.ts',
      },
    },
    dependencies,
  };
}

function materializeMiniProgramPackages() {
  // ---- mp-commons --------------------------------------------------------
  const commonsDir = path.join(mpRoot, 'packages/sdkwork-agents-mp-commons');
  writeJson(path.join(commonsDir, 'package.json'), mpPackageJson(
    'sdkwork-agents-mp-commons',
    '@sdkwork/agents-mp-commons',
    {},
  ));
  writeFile(path.join(commonsDir, 'README.md'), `# sdkwork-agents-mp-commons

Domain-neutral mini program primitives: list/screen state helpers, design
tokens, and i18n helpers. Must not own business pages or SDK construction.

Authority: \`MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md\` section 3.
`);
  writeJson(
    path.join(commonsDir, 'specs/component.spec.json'),
    mpComponentSpec({ packageDirName: 'sdkwork-agents-mp-commons', capability: 'commons', layerRole: 'frontend-commons' }),
  );
  writeFile(path.join(commonsDir, 'src/index.ts'), `export * from "./components/screenStates";
export * from "./theme/designTokens";
export * from "./i18n/locale";
`);
  writeFile(path.join(commonsDir, 'src/theme/designTokens.ts'), `/**
 * Domain-neutral design tokens for the mini program surface.
 *
 * Authority: \`APP_MINI_PROGRAM_UI_SPEC.md\`. Domain-neutral only; business
 * screens belong to capability packages.
 */
export const agentsMpTokens = {
  colorPrimary: "#0f766e",
  colorBackground: "#f8fafc",
  colorSurface: "#ffffff",
  colorText: "#0f172a",
  colorTextMuted: "#64748b",
  colorDanger: "#dc2626",
  spacingSm: "16rpx",
  spacingMd: "32rpx",
  spacingLg: "48rpx",
} as const;

export type AgentsMpTokens = typeof agentsMpTokens;
`);
  writeFile(path.join(commonsDir, 'src/i18n/locale.ts'), `/**
 * Thin locale boundary for the agents mini program capability family.
 *
 * Authority: \`I18N_SPEC.md\` section 6.1. Thin boundaries (\`index\`, \`manifest\`,
 * \`locale\`, \`locales\`, \`registry\`, \`runtime\`, \`types\`, provider) may normalize,
 * look up, register, or type fragments; they MUST NOT author feature copy. The
 * authored fragments live under \`<locale>/<domain>/<capability>/<fragment>.ts\`.
 */
export type AgentsMpLocale = "en-US" | "zh-CN";

export function normalizeAgentsMpLocale(value: string): AgentsMpLocale {
  return value.trim().toLowerCase().startsWith("zh") ? "zh-CN" : "en-US";
}

export function pickAgentsMpMessage(
  fragment: Record<string, string>,
  key: string,
  fallback: string,
): string {
  const value = messages[key];
  return typeof value === "string" && value.length > 0 ? value : fallback;
}
`);
  writeFile(path.join(commonsDir, 'src/components/screenStates.ts'), `/**
 * Domain-neutral screen/list state primitives.
 *
 * Capability packages map these to their own data; the primitives stay
 * payload-free so they remain reusable across capabilities.
 */
export type AgentsMpScreenStatus = "loading" | "ready" | "empty" | "error";

export interface AgentsMpScreenState {
  readonly status: AgentsMpScreenStatus;
  readonly errorMessage?: string;
}

export const initialAgentsMpScreenState: AgentsMpScreenState = { status: "loading" };

export function resolveAgentsMpScreenStatus(itemCount: number, loading: boolean, errorMessage?: string): AgentsMpScreenStatus {
  if (loading) {
    return "loading";
  }
  if (typeof errorMessage === "string" && errorMessage.length > 0) {
    return "error";
  }
  return itemCount === 0 ? "empty" : "ready";
}
`);

  // ---- mp-shell ----------------------------------------------------------
  const shellDir = path.join(mpRoot, 'packages/sdkwork-agents-mp-shell');
  writeJson(path.join(shellDir, 'package.json'), mpPackageJson(
    'sdkwork-agents-mp-shell',
    '@sdkwork/agents-mp-shell',
    {},
  ));
  writeFile(path.join(shellDir, 'README.md'), `# sdkwork-agents-mp-shell

Mini program app shell: page/tab route composition, route projection inputs,
and AuthGate integration. Must not own business services.

Authority: \`MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md\` sections 3, 5, and 9.
`);
  writeJson(
    path.join(shellDir, 'specs/component.spec.json'),
    mpComponentSpec({ packageDirName: 'sdkwork-agents-mp-shell', capability: 'shell', layerRole: 'frontend-shell' }),
  );
  writeFile(path.join(shellDir, 'src/index.ts'), `export * from "./navigation/routePlacement";
export * from "./auth/authGate";
`);
  writeFile(path.join(shellDir, 'src/navigation/routePlacement.ts'), `/**
 * Mini program route placement metadata and projection inputs.
 *
 * Authority: \`MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md\` section 5. SDKWork
 * packages are source/dependency boundaries; platform \`subpackages\` are
 * runtime loading and package-size boundaries. Build tooling projects route
 * contributions into root pages and subpackages.
 */
export interface MiniProgramRoutePlacement {
  readonly rootPackage?: boolean;
  readonly subpackage?: string;
  readonly pagePath: string;
  readonly preload?: boolean;
}

export interface AgentsMpRouteContribution {
  readonly id: string;
  readonly surface: "app";
  readonly domain: string;
  readonly capability: string;
  readonly screen: string;
  readonly titleKey: string;
  readonly auth: "public" | "required";
  readonly permissionHint?: string;
  readonly miniProgram: MiniProgramRoutePlacement;
}

export interface AgentsMpPageProjectionEntry {
  readonly pagePath: string;
  readonly rootPackage: boolean;
  readonly subpackage?: string;
}

/**
 * Projects route contributions into the mini program \`pages\` list and
 * \`subPackages\` descriptor. Route contributions stay the single source; the
 * physical page files are projection targets.
 */
export function projectAgentsMpPages(routes: AgentsMpRouteContribution[]): AgentsMpPageProjectionEntry[] {
  return routes.map((route) => ({
    pagePath: route.miniProgram.pagePath,
    rootPackage: route.miniProgram.rootPackage === true,
    subpackage: route.miniProgram.subpackage,
  }));
}

export function listAgentsMpRootPages(routes: AgentsMpRouteContribution[]): string[] {
  return projectAgentsMpPages(routes)
    .filter((entry) => entry.rootPackage)
    .map((entry) => entry.pagePath);
}
`);
  writeFile(path.join(shellDir, 'src/auth/authGate.ts'), `/**
 * AuthGate integration for the mini program shell.
 *
 * Route guards are shell/runtime responsibilities. Capability packages declare
 * auth mode and permission hints only.
 */
export interface AgentsMpAuthGateDecision {
  readonly allowed: boolean;
  readonly redirectPagePath?: string;
  readonly reason?: string;
}

export function evaluateAgentsMpAuthGate(
  auth: "public" | "required",
  isAuthenticated: boolean,
  loginPagePath = "pages/home/index",
): AgentsMpAuthGateDecision {
  if (auth === "public" || isAuthenticated) {
    return { allowed: true };
  }
  return { allowed: false, redirectPagePath: loginPagePath, reason: "authentication-required" };
}
`);

  // ---- mp-agents (capability) -------------------------------------------
  const agentsDir = path.join(mpRoot, 'packages/sdkwork-agents-mp-agents');
  writeJson(path.join(agentsDir, 'package.json'), mpPackageJson(
    'sdkwork-agents-mp-agents',
    '@sdkwork/agents-mp-agents',
    {
      '@sdkwork/agents-mp-core': 'workspace:*',
      '@sdkwork/agents-mp-commons': 'workspace:*',
      '@sdkwork/agents-mp-shell': 'workspace:*',
    },
  ));
  writeFile(path.join(agentsDir, 'README.md'), `# sdkwork-agents-mp-agents

Agents capability for the mini program: catalog listing, paging, view models,
locale fragments, and route contributions with mini program placement metadata.

SDK access is injected: this package never constructs an SDK client. The
platform page obtains the client from the mini program runtime bundle and hands
it to \`createAgentCatalogService\`.

Authority: \`MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md\` sections 3, 4, and 6.
`);
  writeJson(
    path.join(agentsDir, 'specs/component.spec.json'),
    mpComponentSpec({
      packageDirName: 'sdkwork-agents-mp-agents',
      capability: 'agents',
      layerRole: 'frontend-feature',
      extraContracts: {
        providedPorts: [
          { name: 'agentsMpCatalogService', export: '.' },
          { name: 'agentsMpRoutes', export: '.' },
        ],
        requiredPorts: [
          { name: 'agentsSdkClient', package: '@sdkwork/agents-mp-core', export: './sdk' },
          { name: 'agentsMpCommons', package: '@sdkwork/agents-mp-commons', export: '.' },
          { name: 'agentsMpShell', package: '@sdkwork/agents-mp-shell', export: '.' },
        ],
        sdkDependencies: ['sdkwork-agents-app-sdk'],
      },
    }),
  );
  writeFile(path.join(agentsDir, 'src/index.ts'), `export * from "./types/agentModels";
export * from "./services/AgentCatalogService";
export * from "./state/agentCatalogState";
export * from "./i18n";
export * from "./routes/routeContributions";
`);
  writeFile(path.join(agentsDir, 'src/types/agentModels.ts'), `/**
 * View models, screen models, and route params for the agents capability.
 *
 * API DTOs come from the generated app SDK; this file owns view models only.
 */
export interface AgentsMpCatalogItem {
  readonly id: string;
  readonly name: string;
  readonly description: string;
}

export interface AgentsMpCatalogPage {
  readonly items: AgentsMpCatalogItem[];
  readonly page: number;
  readonly hasMore: boolean;
}

export interface AgentsMpRouteParams {
  readonly agentId?: string;
  readonly sessionId?: string;
}
`);
  writeFile(path.join(agentsDir, 'src/services/AgentCatalogService.ts'), `import { resolveAgentsMpScreenStatus, type AgentsMpScreenState } from "@sdkwork/agents-mp-commons";
import { type SdkworkAgentsAppClient } from "@sdkwork/agents-mp-core/sdk";

import { type AgentsMpCatalogItem, type AgentsMpCatalogPage } from "../types/agentModels";

/**
 * Agents catalog orchestration.
 *
 * The SDK client is injected by the platform page; this service performs record
 * mapping and pagination interpretation only and never constructs transport.
 *
 * Authority: \`MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md\` section 4 (\`services/\`)
 * and \`PAGINATION_SPEC.md\`.
 */

const DEFAULT_PAGE_SIZE = 20;

/**
 * Raw agent record shape as returned by the generated app SDK list operation.
 * Kept structural because the transport DTO is generator-owned.
 */
export interface AgentsMpRawAgentRecord {
  readonly agentId?: string;
  readonly id?: string;
  readonly code?: string;
  readonly displayName?: string;
  readonly description?: string;
}

export function mapAgentsMpCatalogItem(record: AgentsMpRawAgentRecord | null | undefined): AgentsMpCatalogItem | null {
  if (!record || typeof record !== "object") {
    return null;
  }
  const id = record.agentId ?? record.id ?? record.code;
  if (typeof id !== "string" || id.length === 0) {
    return null;
  }
  const name = record.displayName ?? record.code ?? "Agent";
  return {
    id,
    name: typeof name === "string" ? name : String(name),
    description: typeof record.description === "string" ? record.description : "",
  };
}

export function extractAgentsMpItems(response: unknown): AgentsMpRawAgentRecord[] {
  if (!response || typeof response !== "object") {
    return [];
  }
  const candidate = response as { items?: unknown; data?: { items?: unknown } };
  if (Array.isArray(candidate.items)) {
    return candidate.items as AgentsMpRawAgentRecord[];
  }
  if (candidate.data && Array.isArray(candidate.data.items)) {
    return candidate.data.items as AgentsMpRawAgentRecord[];
  }
  return [];
}

export function resolveAgentsMpHasMore(response: unknown): boolean {
  if (!response || typeof response !== "object") {
    return false;
  }
  const candidate = response as {
    pageInfo?: { page?: unknown; totalPages?: unknown; total_pages?: unknown; hasMore?: unknown };
    data?: { pageInfo?: { page?: unknown; totalPages?: unknown; total_pages?: unknown; hasMore?: unknown } };
  };
  const pageInfo = candidate.pageInfo ?? candidate.data?.pageInfo ?? {};
  if (pageInfo.hasMore === true) {
    return true;
  }
  const page = Number(pageInfo.page ?? 1);
  const totalPages = Number(pageInfo.totalPages ?? pageInfo.total_pages ?? 0);
  return totalPages > 0 && page < totalPages;
}

export interface AgentCatalogService {
  loadPage(page: number, pageSize?: number): Promise<AgentsMpCatalogPage>;
}

export function createAgentCatalogService(client: SdkworkAgentsAppClient): AgentCatalogService {
  return {
    async loadPage(page: number, pageSize: number = DEFAULT_PAGE_SIZE): Promise<AgentsMpCatalogPage> {
      if (!Number.isInteger(page) || page < 1) {
        throw new Error("page must be a positive integer");
      }
      if (!Number.isInteger(pageSize) || pageSize < 1) {
        throw new Error("pageSize must be a positive integer");
      }
      const response = await client.ai.agents.list({ page, pageSize });
      const items = extractAgentsMpItems(response)
        .map(mapAgentsMpCatalogItem)
        .filter((item): item is AgentsMpCatalogItem => item !== null);
      return { items, page, hasMore: resolveAgentsMpHasMore(response) };
    },
  };
}

export function resolveAgentsMpCatalogScreenState(
  itemCount: number,
  loading: boolean,
  errorMessage?: string,
): AgentsMpScreenState {
  return {
    status: resolveAgentsMpScreenStatus(itemCount, loading, errorMessage),
    errorMessage,
  };
}
`);
  writeFile(path.join(agentsDir, 'src/state/agentCatalogState.ts'), `/**
 * Package-local state slice for the agents catalog.
 *
 * Sensitive state must clear on logout and account/tenant switch.
 */
export interface AgentsMpCatalogStateSlice {
  readonly page: number;
  readonly items: unknown[];
  readonly hasMore: boolean;
  readonly loading: boolean;
  readonly errorMessage: string;
}

export const initialAgentsMpCatalogState: AgentsMpCatalogStateSlice = {
  page: 1,
  items: [],
  hasMore: false,
  loading: true,
  errorMessage: "",
};

export function clearAgentsMpCatalogState(): AgentsMpCatalogStateSlice {
  return { ...initialAgentsMpCatalogState };
}
`);
  writeFile(path.join(agentsDir, 'src/i18n/en-US/agents/catalog/list.ts'), `/**
 * Authored locale fragment: agents catalog list screen (en-US).
 *
 * Authority: \`I18N_SPEC.md\` section 6.1 — \`src/i18n/<locale>/<domain>/<capability>/<fragment>.ts\`.
 * One fragment maps to one screen so reviews and merges stay conflict-free.
 */
export const agentsMpCatalogListEnUs: Record<string, string> = {
  "agents.catalog.title": "Agents",
  "agents.catalog.loading": "Loading...",
  "agents.catalog.empty": "No agents yet",
  "agents.catalog.loadFailed": "Failed to load agents",
  "agents.catalog.loadMore": "Load more",
};
`);
  writeFile(path.join(agentsDir, 'src/i18n/zh-CN/agents/catalog/list.ts'), `/**
 * Authored locale fragment: agents catalog list screen (zh-CN).
 *
 * Authority: \`I18N_SPEC.md\` section 6.1 — \`src/i18n/<locale>/<domain>/<capability>/<fragment>.ts\`.
 * Key names stay identical across locales; only the copy differs.
 */
export const agentsMpCatalogListZhCn: Record<string, string> = {
  "agents.catalog.title": "智能体",
  "agents.catalog.loading": "加载中...",
  "agents.catalog.empty": "暂无智能体",
  "agents.catalog.loadFailed": "智能体加载失败",
  "agents.catalog.loadMore": "加载更多",
};
`);
  writeFile(path.join(agentsDir, 'src/i18n/index.ts'), `/**
 * Thin i18n aggregation for the agents mini program capability package.
 *
 * Authority: \`I18N_SPEC.md\` section 6.1. \`src/i18n/index.ts\` may import and
 * re-export authored fragments; it MUST NOT author feature copy itself.
 */
import { agentsMpCatalogListEnUs } from "./en-US/agents/catalog/list";
import { agentsMpCatalogListZhCn } from "./zh-CN/agents/catalog/list";

export { agentsMpCatalogListEnUs, agentsMpCatalogListZhCn };

/** Locale-keyed view of the agents catalog fragments. */
export const agentsMpCatalogFragments: Record<string, Record<string, string>> = {
  "en-US": agentsMpCatalogListEnUs,
  "zh-CN": agentsMpCatalogListZhCn,
};
`);
  writeFile(path.join(agentsDir, 'src/routes/routeContributions.ts'), `import { type AgentsMpRouteContribution } from "@sdkwork/agents-mp-shell";

/**
 * Route contributions for the agents capability.
 *
 * Route ids follow \`<surface>.<domain>.<capability>.<screen>\` and stay aligned
 * with the PC, H5, and HarmonyOS roots. Route metadata must not declare HTTP
 * API paths, SDK methods, raw URL constants, or transport details.
 */
export const agentsMpRouteContributions: AgentsMpRouteContribution[] = [
  {
    id: "app.agents.catalog.list",
    surface: "app",
    domain: "agents",
    capability: "catalog",
    screen: "list",
    titleKey: "agents.catalog.title",
    auth: "required",
    permissionHint: "agents.agents.read",
    miniProgram: { rootPackage: true, pagePath: "pages/agents/index" },
  },
  {
    id: "app.agents.catalog.editor",
    surface: "app",
    domain: "agents",
    capability: "catalog",
    screen: "editor",
    titleKey: "agents.catalog.title",
    auth: "required",
    permissionHint: "agents.agents.write",
    miniProgram: { rootPackage: true, pagePath: "pages/agents-h5/index" },
  },
];
`);
}

// ===========================================================================
// Flutter mobile
// ===========================================================================

const flutterRoot = path.join(repoRoot, 'apps', 'sdkwork-agents-flutter-mobile');
const flutterArchSpec = 'FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md';
const flutterUiSpec = {
  file: 'APP_FLUTTER_UI_SPEC.md',
  path: '../../../../../sdkwork-specs/APP_FLUTTER_UI_SPEC.md',
  purpose: 'Flutter UI package rules.',
};

function flutterComponentSpec({ packageName, capability, layerRole, extraContracts = {} }) {
  return {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: packageName,
      displayName: packageName,
      version: '0.1.0',
      type: 'dart-package',
      root: `apps/sdkwork-agents-flutter-mobile/packages/${packageName}`,
      domain: applicationCode,
      capability,
      surface: 'app',
      languages: ['dart'],
      generated: false,
      manifests: ['pubspec.yaml', 'specs/component.spec.json'],
    },
    canonicalSpecs: [{
      file: flutterArchSpec,
      path: '../../../../../sdkwork-specs/' + flutterArchSpec,
      purpose: 'Flutter mobile application root architecture.',
    }, alignmentSpec].concat(
      layerRole === 'frontend-commons' ? [frontendSpec, flutterUiSpec, i18nSpec]
        : layerRole === 'frontend-shell' ? [frontendSpec, flutterUiSpec]
          : [flutterUiSpec, paginationSpec, i18nSpec],
    ),
    contracts: {
      layerRole,
      publicExports: ['.'],
      sdkClients: [],
      sdkDependencies: [],
      dependencyApiExports: [],
      dependencyApiSurfaces: [],
      ...extraContracts,
    },
    verification: {
      commands: [`flutter analyze apps/sdkwork-agents-flutter-mobile/packages/${packageName}`],
    },
  };
}

function flutterPubspec({ packageName, description, dependencies }) {
  const dependencyLines = Object.entries(dependencies)
    .map(([name, spec]) => `  ${name}:\n    ${spec}`)
    .join('\n');
  return `name: ${packageName}
description: ${description}
version: 0.1.0
publish_to: none

environment:
  sdk: ">=3.5.0 <4.0.0"

dependencies:
  flutter:
    sdk: flutter
${dependencyLines}

dev_dependencies:
  flutter_test:
    sdk: flutter
`;
}

function materializeFlutterPackages() {
  // ---- commons -----------------------------------------------------------
  const commonsDir = path.join(flutterRoot, 'packages/sdkwork_agents_flutter_mobile_commons');
  writeFile(path.join(commonsDir, 'pubspec.yaml'), flutterPubspec({
    packageName: 'sdkwork_agents_flutter_mobile_commons',
    description: 'SDKWork Agents Flutter mobile commons package',
    dependencies: {},
  }));
  writeFile(path.join(commonsDir, 'README.md'), `# sdkwork_agents_flutter_mobile_commons

Domain-neutral Flutter widgets, design tokens, list/screen state primitives, and
i18n helpers. Must not own business screens or SDK construction.

Authority: \`FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 3.
`);
  writeJson(
    path.join(commonsDir, 'specs/component.spec.json'),
    flutterComponentSpec({
      packageName: 'sdkwork_agents_flutter_mobile_commons',
      capability: 'commons',
      layerRole: 'frontend-commons',
    }),
  );
  writeFile(path.join(commonsDir, 'lib/sdkwork_agents_flutter_mobile_commons.dart'), `library sdkwork_agents_flutter_mobile_commons;

export 'src/theme/design_tokens.dart';
export 'src/widgets/screen_states.dart';
export 'src/copy/locale_helpers.dart';
`);
  writeFile(path.join(commonsDir, 'lib/src/theme/design_tokens.dart'), `/// Domain-neutral design tokens for the SDKWork Agents Flutter mobile root.
///
/// Authority: \`APP_FLUTTER_UI_SPEC.md\`. Shared packages must not import
/// application app shells.
class SdkworkAgentsFlutterTokens {
  const SdkworkAgentsFlutterTokens._();

  static const int colorPrimary = 0xFF0F766E;
  static const int colorBackground = 0xFFF8FAFC;
  static const int colorText = 0xFF0F172A;
  static const int colorTextMuted = 0xFF64748B;

  static const double spacingSm = 8;
  static const double spacingMd = 16;
  static const double spacingLg = 24;
}
`);
  writeFile(path.join(commonsDir, 'lib/src/widgets/screen_states.dart'), `import 'package:flutter/material.dart';

/// Domain-neutral screen/list state primitives.
///
/// Capability packages map their own data onto these payload-free primitives.
enum SdkworkAgentsScreenStatus { loading, ready, empty, error }

SdkworkAgentsScreenStatus resolveSdkworkAgentsScreenStatus(
  int itemCount,
  bool loading,
  String? errorMessage,
) {
  if (loading) {
    return SdkworkAgentsScreenStatus.loading;
  }
  if (errorMessage != null && errorMessage.isNotEmpty) {
    return SdkworkAgentsScreenStatus.error;
  }
  return itemCount == 0 ? SdkworkAgentsScreenStatus.empty : SdkworkAgentsScreenStatus.ready;
}

/// Payload-free status widget used by capability screens.
class SdkworkAgentsStatusView extends StatelessWidget {
  const SdkworkAgentsStatusView({super.key, required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Text(message, textAlign: TextAlign.center),
      ),
    );
  }
}
`);
  writeFile(path.join(commonsDir, 'lib/src/copy/locale_helpers.dart'), `/// Thin locale helpers for the agents Flutter mobile capability family.
///
/// Placement note (\`I18N_SPEC.md\` section 6.1): the authored Flutter fragment
/// layout is \`lib/src/i18n/<locale>/<domain>/<capability>/<screen-or-widget>.arb\`
/// or \`.json\` — \`.dart\` is not an authored fragment extension. Dart code that
/// normalizes a locale or looks a key up in an already-loaded fragment is a
/// boundary helper, not a locale resource, so it lives outside \`lib/src/i18n/\`.
/// The \`gen_l10n\` projection that generates Dart accessors from \`.arb\` fragments
/// requires the Flutter toolchain and is tracked as pending integration.
enum SdkworkAgentsLocale { enUs, zhCn }

SdkworkAgentsLocale normalizeSdkworkAgentsLocale(String value) {
  final normalized = value.trim().toLowerCase();
  return normalized.startsWith('zh') ? SdkworkAgentsLocale.zhCn : SdkworkAgentsLocale.enUs;
}

String pickSdkworkAgentsMessage(
  Map<String, String> messages,
  String key,
  String fallback,
) {
  final value = messages[key];
  return value != null && value.isNotEmpty ? value : fallback;
}
`);

  // ---- shell -------------------------------------------------------------
  const shellDir = path.join(flutterRoot, 'packages/sdkwork_agents_flutter_mobile_shell');
  writeFile(path.join(shellDir, 'pubspec.yaml'), flutterPubspec({
    packageName: 'sdkwork_agents_flutter_mobile_shell',
    description: 'SDKWork Agents Flutter mobile shell package',
    dependencies: {
      sdkwork_agents_flutter_mobile_core: 'path: ../sdkwork_agents_flutter_mobile_core',
    },
  }));
  writeFile(path.join(shellDir, 'README.md'), `# sdkwork_agents_flutter_mobile_shell

Flutter app shell: router/navigation assembly, tab/stack composition, and
AuthGate integration. Must not own business services.

Authority: \`FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 3.
`);
  writeJson(
    path.join(shellDir, 'specs/component.spec.json'),
    flutterComponentSpec({
      packageName: 'sdkwork_agents_flutter_mobile_shell',
      capability: 'shell',
      layerRole: 'frontend-shell',
    }),
  );
  writeFile(path.join(shellDir, 'lib/sdkwork_agents_flutter_mobile_shell.dart'), `library sdkwork_agents_flutter_mobile_shell;

export 'src/navigation/route_registry.dart';
export 'src/auth/auth_gate.dart';
`);
  writeFile(path.join(shellDir, 'lib/src/navigation/route_registry.dart'), `/// Named-route registry assembly for the Flutter mobile shell.
///
/// Route ids follow \`<surface>.<domain>.<capability>.<screen>\` and stay
/// aligned with the PC, H5, mini program, and HarmonyOS roots. Physical Flutter
/// route names may differ while route ids stay stable.
class SdkworkAgentsRouteRegistration {
  const SdkworkAgentsRouteRegistration({
    required this.id,
    required this.routeName,
    required this.titleKey,
    required this.authRequired,
  });

  final String id;
  final String routeName;
  final String titleKey;
  final bool authRequired;
}

List<SdkworkAgentsRouteRegistration> createSdkworkAgentsRouteRegistry(
  List<SdkworkAgentsRouteRegistration> routes,
) {
  return routes.where((route) => route.routeName.isNotEmpty).toList(growable: false);
}
`);
  writeFile(path.join(shellDir, 'lib/src/auth/auth_gate.dart'), `/// AuthGate integration for the Flutter mobile shell.
///
/// Route guards are shell/runtime responsibilities. Capability packages declare
/// auth mode and permission hints only.
class SdkworkAgentsAuthGateDecision {
  const SdkworkAgentsAuthGateDecision({required this.allowed, this.reason});

  final bool allowed;
  final String? reason;
}

SdkworkAgentsAuthGateDecision evaluateSdkworkAgentsAuthGate({
  required bool authRequired,
  required bool isAuthenticated,
}) {
  if (!authRequired || isAuthenticated) {
    return const SdkworkAgentsAuthGateDecision(allowed: true);
  }
  return const SdkworkAgentsAuthGateDecision(
    allowed: false,
    reason: 'authentication-required',
  );
}
`);

  // ---- agents capability -------------------------------------------------
  const agentsDir = path.join(flutterRoot, 'packages/sdkwork_agents_flutter_mobile_agents');
  writeFile(path.join(agentsDir, 'pubspec.yaml'), flutterPubspec({
    packageName: 'sdkwork_agents_flutter_mobile_agents',
    description: 'SDKWork Agents Flutter mobile agents capability package',
    dependencies: {
      sdkwork_agents_flutter_mobile_core: 'path: ../sdkwork_agents_flutter_mobile_core',
      sdkwork_agents_flutter_mobile_commons: 'path: ../sdkwork_agents_flutter_mobile_commons',
      sdkwork_agents_flutter_mobile_shell: 'path: ../sdkwork_agents_flutter_mobile_shell',
      sdkwork_agents_app_sdk: 'path: ../../../../sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-flutter/generated/server-openapi',
    },
  }));
  writeFile(path.join(agentsDir, 'README.md'), `# sdkwork_agents_flutter_mobile_agents

Agents capability for the Flutter mobile root: catalog screens, widgets,
controllers, services, view models, locale fragments, and route contributions.

SDK access is injected: this package never constructs an SDK client. The root
bootstrap creates \`AgentsAppSdkClients\` and hands the typed client to
\`AgentCatalogService\`.

Authority: \`FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md\` sections 3 and 4.
`);
  writeJson(
    path.join(agentsDir, 'specs/component.spec.json'),
    flutterComponentSpec({
      packageName: 'sdkwork_agents_flutter_mobile_agents',
      capability: 'agents',
      layerRole: 'frontend-feature',
      extraContracts: {
        providedPorts: [
          { name: 'agentsFlutterCatalogScreens', export: '.' },
          { name: 'agentsFlutterRoutes', export: '.' },
        ],
        requiredPorts: [
          { name: 'agentsAppSdkClients', package: 'sdkwork_agents_flutter_mobile_core', export: '.' },
          { name: 'agentsFlutterCommons', package: 'sdkwork_agents_flutter_mobile_commons', export: '.' },
        ],
        sdkClients: ['SdkworkAppClient'],
        sdkDependencies: ['sdkwork-agents-app-sdk'],
      },
    }),
  );
  writeFile(path.join(agentsDir, 'lib/sdkwork_agents_flutter_mobile_agents.dart'), `library sdkwork_agents_flutter_mobile_agents;

export 'src/models/agent_models.dart';
export 'src/services/agent_catalog_service.dart';
export 'src/controllers/agents_catalog_controller.dart';
export 'src/state/agent_catalog_state.dart';
export 'src/copy/agents_messages.dart';
export 'src/routes/route_contributions.dart';
export 'src/screens/agents_catalog_screen.dart';
`);
  writeFile(path.join(agentsDir, 'lib/src/models/agent_models.dart'), `/// View models, screen models, and route params for the agents capability.
///
/// API DTOs come from the generated Dart app SDK; this file owns view models
/// only.
class AgentsCatalogItem {
  const AgentsCatalogItem({
    required this.id,
    required this.name,
    required this.description,
  });

  final String id;
  final String name;
  final String description;
}

class AgentsCatalogPage {
  const AgentsCatalogPage({
    required this.items,
    required this.page,
    required this.hasMore,
  });

  final List<AgentsCatalogItem> items;
  final int page;
  final bool hasMore;
}

class AgentsRouteParams {
  const AgentsRouteParams({required this.agentId, this.sessionId});

  final String agentId;
  final String? sessionId;
}
`);
  writeFile(path.join(agentsDir, 'lib/src/services/agent_catalog_service.dart'), `import 'package:sdkwork_agents_app_sdk/sdkwork_agents_app_sdk.dart';

import '../models/agent_models.dart';

/// Agents catalog orchestration.
///
/// The SDK client is injected by root bootstrap; this service maps records and
/// interprets pagination only and never constructs transport.
///
/// Authority: \`FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 4
/// (\`services/\`) and \`PAGINATION_SPEC.md\`.
const int defaultAgentsCatalogPageSize = 20;

class AgentCatalogService {
  const AgentCatalogService({required this.client});

  final SdkworkAppClient client;

  Future<AgentsCatalogPage> loadPage({
    required int page,
    int pageSize = defaultAgentsCatalogPageSize,
  }) async {
    if (page < 1) {
      throw ArgumentError.value(page, 'page', 'must be a positive integer');
    }
    if (pageSize < 1) {
      throw ArgumentError.value(pageSize, 'pageSize', 'must be a positive integer');
    }
    final response = await client.ai.agents.list(
      page: page,
      pageSize: pageSize,
    );
    final items = extractAgentsCatalogItems(response);
    return AgentsCatalogPage(
      items: items,
      page: page,
      hasMore: resolveAgentsCatalogHasMore(response),
    );
  }
}

/// Structural mapping because the transport DTO is generator-owned.
List<AgentsCatalogItem> extractAgentsCatalogItems(Object? response) {
  final records = _extractRecords(response);
  final items = <AgentsCatalogItem>[];
  for (final record in records) {
    final mapped = _mapRecord(record);
    if (mapped != null) {
      items.add(mapped);
    }
  }
  return items;
}

List<Object?> _extractRecords(Object? response) {
  if (response is Map) {
    final items = response['items'];
    if (items is List) {
      return items;
    }
    final data = response['data'];
    if (data is Map && data['items'] is List) {
      return data['items'] as List;
    }
  }
  return const <Object?>[];
}

AgentsCatalogItem? _mapRecord(Object? record) {
  if (record is! Map) {
    return null;
  }
  final id = record['agentId'] ?? record['id'] ?? record['code'];
  if (id is! String || id.isEmpty) {
    return null;
  }
  final name = record['displayName'] ?? record['code'] ?? 'Agent';
  final description = record['description'];
  return AgentsCatalogItem(
    id: id,
    name: name is String ? name : name.toString(),
    description: description is String ? description : '',
  );
}

bool resolveAgentsCatalogHasMore(Object? response) {
  if (response is! Map) {
    return false;
  }
  Map<Object?, Object?> pageInfo = const <Object?, Object?>{};
  final direct = response['pageInfo'];
  if (direct is Map) {
    pageInfo = direct;
  } else {
    final data = response['data'];
    if (data is Map && data['pageInfo'] is Map) {
      pageInfo = data['pageInfo'] as Map;
    }
  }
  if (pageInfo['hasMore'] == true) {
    return true;
  }
  final page = int.tryParse((pageInfo['page'] ?? 1).toString()) ?? 1;
  final totalPages = int.tryParse(
        (pageInfo['totalPages'] ?? pageInfo['total_pages'] ?? 0).toString(),
      ) ??
      0;
  return totalPages > 0 && page < totalPages;
}
`);
  writeFile(path.join(agentsDir, 'lib/src/controllers/agents_catalog_controller.dart'), `import 'package:flutter/foundation.dart';

import '../models/agent_models.dart';
import '../services/agent_catalog_service.dart';
import '../state/agent_catalog_state.dart';

/// Presentation controller for the agents catalog.
///
/// Owns UI state mapping and calls services only.
class AgentsCatalogController extends ChangeNotifier {
  AgentsCatalogController({required this.service});

  final AgentCatalogService service;
  AgentsCatalogState _state = initialAgentsCatalogState();

  AgentsCatalogState get state => _state;

  Future<void> load({int page = 1}) async {
    _state = _state.copyWith(loading: true, errorMessage: '');
    notifyListeners();
    try {
      final result = await service.loadPage(page: page);
      _state = _state.copyWith(
        items: result.items,
        page: result.page,
        hasMore: result.hasMore,
        loading: false,
        errorMessage: '',
      );
    } catch (error) {
      _state = _state.copyWith(
        items: const <AgentsCatalogItem>[],
        loading: false,
        errorMessage: error.toString(),
      );
    }
    notifyListeners();
  }
}
`);
  writeFile(path.join(agentsDir, 'lib/src/state/agent_catalog_state.dart'), `import '../models/agent_models.dart';

/// Package-local state slice for the agents catalog.
///
/// Sensitive state must clear on logout and account/tenant switch.
class AgentsCatalogState {
  const AgentsCatalogState({
    required this.page,
    required this.items,
    required this.hasMore,
    required this.loading,
    required this.errorMessage,
  });

  final int page;
  final List<AgentsCatalogItem> items;
  final bool hasMore;
  final bool loading;
  final String errorMessage;

  AgentsCatalogState copyWith({
    int? page,
    List<AgentsCatalogItem>? items,
    bool? hasMore,
    bool? loading,
    String? errorMessage,
  }) {
    return AgentsCatalogState(
      page: page ?? this.page,
      items: items ?? this.items,
      hasMore: hasMore ?? this.hasMore,
      loading: loading ?? this.loading,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

AgentsCatalogState initialAgentsCatalogState() {
  return const AgentsCatalogState(
    page: 1,
    items: <AgentsCatalogItem>[],
    hasMore: false,
    loading: true,
    errorMessage: '',
  );
}
`);
  writeFile(path.join(agentsDir, 'lib/src/copy/agents_messages.dart'), `/// Package-local default copy for the agents capability.
///
/// Placement note (\`I18N_SPEC.md\` section 6.1): Flutter authored fragments use
/// \`.arb\`/\`.json\` under \`lib/src/i18n/<locale>/<domain>/<capability>/\`. Dart
/// \`const\` maps are not an authored fragment format, so they are classified as
/// code-level defaults and live outside \`lib/src/i18n/\` until the \`gen_l10n\`
/// projection (which needs the Flutter toolchain) replaces them.
const Map<String, String> agentsMessagesEnUs = <String, String>{
  'agents.catalog.title': 'Agents',
  'agents.catalog.loading': 'Loading...',
  'agents.catalog.empty': 'No agents yet',
  'agents.catalog.loadFailed': 'Failed to load agents',
};

const Map<String, String> agentsMessagesZhCn = <String, String>{
  'agents.catalog.title': '智能体',
  'agents.catalog.loading': '加载中...',
  'agents.catalog.empty': '暂无智能体',
  'agents.catalog.loadFailed': '智能体加载失败',
};
`);
  writeFile(path.join(agentsDir, 'lib/src/routes/route_contributions.dart'), `import 'package:sdkwork_agents_flutter_mobile_shell/sdkwork_agents_flutter_mobile_shell.dart';

/// Route contributions for the agents capability.
///
/// Route ids follow \`<surface>.<domain>.<capability>.<screen>\` and stay
/// aligned with the PC, H5, mini program, and HarmonyOS roots. Route metadata
/// must not declare HTTP API paths, SDK methods, or transport details.
const List<SdkworkAgentsRouteRegistration> agentsRouteContributions =
    <SdkworkAgentsRouteRegistration>[
  SdkworkAgentsRouteRegistration(
    id: 'app.agents.catalog.list',
    routeName: '/agents',
    titleKey: 'agents.catalog.title',
    authRequired: true,
  ),
  SdkworkAgentsRouteRegistration(
    id: 'app.agents.conversation.chat',
    routeName: '/agents/:agentId/chat',
    titleKey: 'agents.catalog.title',
    authRequired: true,
  ),
];
`);
  writeFile(path.join(agentsDir, 'lib/src/screens/agents_catalog_screen.dart'), `import 'package:flutter/material.dart';
import 'package:sdkwork_agents_flutter_mobile_commons/sdkwork_agents_flutter_mobile_commons.dart';

import '../copy/agents_messages.dart';
import '../models/agent_models.dart';
import '../state/agent_catalog_state.dart';

/// Route-level capability screen. Root shell mounts this; business UI lives in
/// capability packages.
class AgentsCatalogScreen extends StatelessWidget {
  const AgentsCatalogScreen({super.key, required this.state});

  final AgentsCatalogState state;

  @override
  Widget build(BuildContext context) {
    final status = resolveSdkworkAgentsScreenStatus(
      state.items.length,
      state.loading,
      state.errorMessage,
    );
    if (status != SdkworkAgentsScreenStatus.ready) {
      return Scaffold(
        appBar: AppBar(title: const Text('Agents')),
        body: SdkworkAgentsStatusView(
          message: switch (status) {
            SdkworkAgentsScreenStatus.loading =>
              agentsMessagesEnUs['agents.catalog.loading']!,
            SdkworkAgentsScreenStatus.empty =>
              agentsMessagesEnUs['agents.catalog.empty']!,
            SdkworkAgentsScreenStatus.error =>
              agentsMessagesEnUs['agents.catalog.loadFailed']!,
            SdkworkAgentsScreenStatus.ready => '',
          },
        ),
      );
    }
    return Scaffold(
      appBar: AppBar(title: const Text('Agents')),
      body: ListView.builder(
        itemCount: state.items.length,
        itemBuilder: (context, index) {
          final AgentsCatalogItem item = state.items[index];
          return ListTile(
            title: Text(item.name),
            subtitle: item.description.isEmpty ? null : Text(item.description),
          );
        },
      ),
    );
  }
}
`);
}

materializeMiniProgramPackages();
materializeFlutterPackages();

console.log('materialize-client-package-families');
console.log(`  created: ${created.length}`);
console.log(`  skipped (already present): ${skipped.length}`);
for (const relativePath of created) {
  console.log(`    + ${relativePath}`);
}
