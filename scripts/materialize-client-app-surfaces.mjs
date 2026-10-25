#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const applicationCode = 'agents';
const backendAppId = 'sdkwork-agents';
const defaultTenantId = '100001';
const defaultOrganizationId = '0';
const defaultPublicHttpUrl = 'http://127.0.0.1:8095';
const defaultPlatformGatewayHttpUrl = 'http://127.0.0.1:3900';
const defaultAppbaseGatewayHttpUrl = defaultPublicHttpUrl;

function write(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  if (fs.existsSync(filePath)) {
    return false;
  }
  fs.writeFileSync(filePath, content);
  return true;
}

function writeJson(filePath, value) {
  return write(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function writeReadme(filePath, content) {
  return write(filePath, content);
}

function manifestV3({
  key,
  displayName,
  description,
  appType,
  appId,
  platform,
  framework,
  family,
  runtimes,
  defaultPlatform,
  sourceRoot,
  packageName = null,
  bundleId = null,
}) {
  return {
    schemaVersion: 3,
    kind: 'agent.app',
    app: {
      key,
      name: displayName,
      displayName,
      description,
      vendor: 'SDKWork',
      officialWebsiteUrl: `https://sdkwork.com/apps/${key}`,
      supportUrl: 'https://sdkwork.com/support',
      privacyPolicyUrl: 'https://sdkwork.com/privacy',
      termsOfServiceUrl: 'https://sdkwork.com/terms',
      appType,
      versionSource: appType === 'APP_FLUTTER' ? 'pubspec.yaml' : 'package.json',
      identifiers: {
        packageName,
        bundleId,
        desktopAppId: appType === 'APP_REACT' && family === 'web' ? `com.sdkwork.${applicationCode}.pc.browser` : null,
        containerImage: `registry.sdkwork.com/apps/${key}`,
      },
    },
    backend: {
      profileKey: 'backend-root-admin',
      ownerMode: 'tenant',
      grantMode: 'current',
      platform,
      appId,
      organizationId: defaultOrganizationId,
      tenantId: defaultTenantId,
      accessTokenPermissionScope: [
        'iam.users.read',
        'iam.organizations.read',
        'iam.roles.read',
        'iam.permissions.read',
      ],
    },
    runtime: {
      family,
      framework,
      runtimes,
      deliveryModes: appType === 'APP_FLUTTER' ? ['DIRECT_DOWNLOAD'] : ['WEB_URL'],
      defaultPlatform,
      defaultArchitecture: 'universal',
      supportedDeploymentProfiles: ['standalone', 'cloud'],
      defaultDeploymentProfile: 'standalone',
    },
    media: {
      icons: {
        primary: {
          id: `${key}-primary-icon`,
          type: 'ICON',
          purpose: 'PRIMARY',
          platform: 'APP',
          locale: 'en-US',
          width: 1024,
          height: 1024,
          format: 'PNG',
          enabled: true,
          metadata: { generatedPlaceholder: true },
        },
        platform: [],
        metadata: { generatedPlaceholder: true },
      },
      screenshots: [],
      previews: [],
      metadata: { assetVersion: '0.1.0', defaultLocale: 'en-US' },
    },
    publish: {
      status: 'BETA',
      installSkill: false,
      platforms: runtimes,
      installPlatforms: runtimes,
      config: {
        workspaceRoot: sourceRoot,
        framework,
        managedBy: 'sdkwork-agents-client-scaffold',
      },
    },
    artifacts: {
      installConfig: {
        packages: [],
        metadata: {
          workspaceRoot: sourceRoot,
          framework,
          packageManager: appType === 'APP_FLUTTER' ? 'flutter' : 'pnpm',
        },
      },
    },
    release: {
      currentVersion: '0.1.0',
      defaultChannel: 'BETA',
      latest: { BETA: '0.1.0' },
      notes: [],
    },
    security: {
      checksumRequired: true,
      signatureRequired: false,
      sbomRequired: true,
    },
    devApp: {
      build: { targets: [] },
      sourceRoot,
    },
    metadata: {
      standardOwner: 'sdkwork-agents',
      initializedAt: new Date().toISOString(),
    },
  };
}

function agentsMd(surface, archSpec, appId, verifyBlock) {
  return `# SDKWork Agents ${surface} Application

## Entry Point

This is the ${surface} application root for SDKWork Agents. See [../../AGENTS.md](../../AGENTS.md) for repository-level agent instructions.

## SDKWork Specs

- \`../../sdkwork-specs/README.md\`
- \`../../sdkwork-specs/SOUL.md\`
- \`../../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md\`
- \`../../sdkwork-specs/${archSpec}\`
- \`../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md\`
- \`../../sdkwork-specs/CONFIG_SPEC.md\`

## Application Identity

- App ID: \`${appId}\`
- Application code: \`${applicationCode}\`
- Runtime family: client surface for SDKWork Agents

## Build And Verify

${verifyBlock}
`;
}

function sdkworkDotSdkworkReadme() {
  return `# SDKWork Application Workspace

Local skills and plugins for this client application root follow \`SDKWORK_WORKSPACE_SPEC.md\`.
`;
}

function createTsPackage(appRoot, packageDirName, npmName, capability = 'core') {
  const packageDir = path.join(appRoot, 'packages', packageDirName);
  writeJson(path.join(packageDir, 'package.json'), {
    name: npmName,
    private: true,
    version: '0.1.0',
    type: 'module',
    exports: capability === 'core'
      ? {
          '.': './src/index.ts',
          './sdk': './src/sdk/index.ts',
          './modules': './src/modules/index.ts',
          './host': './src/host/index.ts',
          './session': './src/session/index.ts',
          './composition': './src/composition/index.ts',
        }
      : {
          '.': './src/index.ts',
        },
  });
  write(path.join(packageDir, 'src/index.ts'), 'export {};\n');
  if (capability === 'core') {
    for (const sub of ['sdk', 'modules', 'host', 'session']) {
      write(path.join(packageDir, 'src', sub, 'index.ts'), 'export {};\n');
    }
  }
  writeJson(path.join(packageDir, 'specs/component.spec.json'), {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: packageDirName,
      displayName: packageDirName,
      version: '0.1.0',
      type: 'typescript-package',
      root: `apps/${path.basename(appRoot)}/packages/${packageDirName}`,
      domain: applicationCode,
      capability,
      surface: 'app',
      languages: ['typescript'],
      generated: false,
      manifests: ['package.json'],
    },
    contracts: {
      publicExports: ['.'],
      sdkClients: capability === 'core' ? ['sdkwork-agents-app-sdk'] : [],
      sdkDependencies: capability === 'core'
        ? [{ workspace: 'sdkwork-agents-app-sdk', surface: 'app-api', credentialMode: 'authenticated-app-api' }]
        : [],
    },
  });
}

function createReactSurface({
  suffix,
  archId,
  archSpec,
  appType,
  platform,
  framework,
  family,
  runtimes,
  defaultPlatform,
  vitePort,
  envPrefix,
  packageSegment,
  extraPackages = [],
}) {
  const appRootName = `sdkwork-${applicationCode}-${suffix}`;
  const appRoot = path.join(repoRoot, 'apps', appRootName);
  const manifestKey = `${applicationCode}-${suffix}`;
  const appId = backendAppId;

  write(path.join(appRoot, 'AGENTS.md'), agentsMd(
    suffix.toUpperCase(),
    archSpec,
    appId,
    '```powershell\npnpm install\npnpm run typecheck\npnpm run build\n```',
  ));
  writeJson(path.join(appRoot, 'agent.app.config.json'), manifestV3({
    key: manifestKey,
    displayName: `SDKWork Agents ${suffix.toUpperCase()}`,
    description: `SDKWork Agents ${suffix} client application scaffold.`,
    appType,
    appId,
    platform,
    framework,
    family,
    runtimes,
    defaultPlatform,
    sourceRoot: `apps/${appRootName}`,
  }));
  writeReadme(path.join(appRoot, '.sdkwork/README.md'), sdkworkDotSdkworkReadme());
  writeReadme(path.join(appRoot, '.sdkwork/skills/README.md'), '# Skills\n');
  writeReadme(path.join(appRoot, '.sdkwork/plugins/README.md'), '# Plugins\n');

  writeJson(path.join(appRoot, 'specs/component.spec.json'), {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: appRootName,
      displayName: `SDKWork Agents ${suffix.toUpperCase()}`,
      version: '0.1.0',
      type: `${archId}-app-root`,
      root: `apps/${appRootName}`,
      domain: applicationCode,
      capability: applicationCode,
      surface: 'app',
      languages: ['typescript'],
      generated: false,
      manifests: ['package.json', 'agent.app.config.json'],
    },
    canonicalSpecs: [
      {
        file: archSpec,
        path: `../../../../sdkwork-specs/${archSpec}`,
        purpose: `${suffix} application root architecture standard.`,
      },
      {
        file: 'APP_SDK_INTEGRATION_SPEC.md',
        path: '../../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md',
        purpose: 'SDK integration and TokenManager wiring.',
      },
    ],
    contracts: {
      publicExports: ['src/main.tsx'],
      runtimeEntrypoints: ['package.json#scripts.dev'],
      sdkClients: [],
      sdkDependencies: ['sdkwork-agents-app-sdk'],
      dependencyApiExports: [],
      dependencyApiSurfaces: [],
    },
    verification: {
      commands: ['pnpm run typecheck', 'pnpm run build'],
    },
  });

  writeJson(path.join(appRoot, 'config/browser/runtime-env.development.example.json'), {
    agents: {
      apiBaseUrl: `${defaultPublicHttpUrl}/app/v3/api`,
      backendApiBaseUrl: `${defaultPublicHttpUrl}/backend/v3/api`,
    },
    appbase: {
      appApiBaseUrl: defaultAppbaseGatewayHttpUrl,
      loginUrl: defaultAppbaseGatewayHttpUrl,
    },
  });

  write(path.join(appRoot, '.env.example'), [
    '# Private bootstrap credential for protected app-api/backend-api before interactive login.',
    'SDKWORK_ACCESS_TOKEN=',
    '',
    `# Browser-visible runtime overrides for ${appRootName}.`,
    `${envPrefix}_ENVIRONMENT=development`,
    `${envPrefix}_APPLICATION_PUBLIC_HTTP_URL=${defaultPublicHttpUrl}`,
    `${envPrefix}_APP_API_BASE_URL=${defaultPublicHttpUrl}/app/v3/api`,
    'VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL=' + defaultPlatformGatewayHttpUrl,
    envPrefix + '_APPBASE_APP_API_BASE_URL=' + defaultAppbaseGatewayHttpUrl,
    `${envPrefix}_BACKEND_API_BASE_URL=${defaultPublicHttpUrl}/backend/v3/api`,
    envPrefix + '_APPBASE_LOGIN_URL=' + defaultAppbaseGatewayHttpUrl,
    '',
  ].join('\n'));

  write(path.join(appRoot, 'index.html'), `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>SDKWork Agents ${suffix.toUpperCase()}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
`);

  writeJson(path.join(appRoot, 'package.json'), {
    name: `@sdkwork/${applicationCode}-${packageSegment}`,
    private: true,
    version: '0.1.0',
    type: 'module',
    scripts: {
      dev: 'vite',
      build: 'tsc -p tsconfig.app.json && vite build',
      preview: 'vite preview',
      typecheck: 'tsc --noEmit -p tsconfig.app.json',
    },
    dependencies: {
      [`@sdkwork/${applicationCode}-${packageSegment}-core`]: 'workspace:*',
      [`@sdkwork/${applicationCode}-${packageSegment}-commons`]: 'workspace:*',
      [`@sdkwork/${applicationCode}-${packageSegment}-shell`]: 'workspace:*',
      '@sdkwork/sdk-common': 'workspace:*',
      react: 'catalog:',
      'react-dom': 'catalog:',
      'react-router-dom': '^7.17.0',
    },
    devDependencies: {
      '@tailwindcss/vite': 'catalog:',
      '@types/react': 'catalog:',
      '@types/react-dom': 'catalog:',
      '@vitejs/plugin-react': 'catalog:',
      tailwindcss: 'catalog:',
      typescript: 'catalog:',
      vite: 'catalog:',
    },
  });

  write(path.join(appRoot, 'pnpm-workspace.yaml'), `packages:
  - "packages/*"
`);

  writeJson(path.join(appRoot, 'tsconfig.json'), {
    compilerOptions: {
      target: 'ES2022',
      module: 'ESNext',
      moduleResolution: 'bundler',
      jsx: 'react-jsx',
      strict: true,
      esModuleInterop: true,
      skipLibCheck: true,
      forceConsistentCasingInFileNames: true,
      allowImportingTsExtensions: true,
      noEmit: true,
    },
    include: ['src'],
  });

  writeJson(path.join(appRoot, 'tsconfig.app.json'), {
    extends: './tsconfig.json',
    compilerOptions: {
      paths: {
        [`@sdkwork/${applicationCode}-${packageSegment}-core`]: [`./packages/sdkwork-${applicationCode}-${packageSegment}-core/src/index.ts`],
        [`@sdkwork/${applicationCode}-${packageSegment}-commons`]: [`./packages/sdkwork-${applicationCode}-${packageSegment}-commons/src/index.ts`],
        [`@sdkwork/${applicationCode}-${packageSegment}-shell`]: [`./packages/sdkwork-${applicationCode}-${packageSegment}-shell/src/index.ts`],
        '@sdkwork/agents-app-sdk': ['../../sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/src/index.ts'],
      },
    },
    include: ['src', `packages/sdkwork-${applicationCode}-${packageSegment}-core/src`, 'src/vite-env.d.ts'],
  });

  const agentsRoot = repoRoot.replaceAll('\\', '/');
  write(path.join(appRoot, 'vite.config.ts'), `import path from "node:path";
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";

const appRoot = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(appRoot, "../..");

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, appRoot, "");
  return {
    define: {
      "process.env.SDKWORK_ACCESS_TOKEN": JSON.stringify(env.SDKWORK_ACCESS_TOKEN ?? ""),
    },
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        "@sdkwork/agents-app-sdk": path.resolve(
          repoRoot,
          "sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/src/index.ts",
        ),
      },
    },
    server: { port: ${vitePort} },
  };
});
`);

  write(path.join(appRoot, 'src/main.tsx'), `import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
`);

  write(path.join(appRoot, 'src/vite-env.d.ts'), `/// <reference types="vite/client" />

declare module "*.css" {}
`);

  write(path.join(appRoot, 'src/index.css'), `:root {
  color-scheme: light;
  font-family: Inter, system-ui, sans-serif;
}

body {
  margin: 0;
  min-height: 100vh;
  background: #f8fafc;
  color: #0f172a;
}
`);

  write(path.join(appRoot, 'src/App.tsx'), `import { HashRouter } from "react-router-dom";

import { bootstrap } from "./bootstrap/runtime";

bootstrap();

export default function App() {
  return (
    <HashRouter>
      <main style={{ padding: "2rem" }}>
        <h1>SDKWork Agents ${suffix.toUpperCase()}</h1>
        <p>Client application scaffold is ready for feature packages and route contributions.</p>
      </main>
    </HashRouter>
  );
}
`);

  const environmentTs = `export type AgentsLifecycleEnvironment = "development" | "test" | "staging" | "production";
export type AgentsRuntimeEnvironment = Readonly<Record<string, unknown>>;

export interface AgentsEnvironment {
  apiBaseUrl: string;
  appbaseAppApiBaseUrl: string;
  backendApiBaseUrl: string;
  appbaseLoginUrl: string;
  lifecycleEnvironment: AgentsLifecycleEnvironment;
}

const APP_API_SUFFIX = "/app/v3/api";
const DEVELOPMENT_APPLICATION_PUBLIC_HTTP_URL = "${defaultPublicHttpUrl}";
const DEVELOPMENT_APPBASE_GATEWAY_HTTP_URL = "${defaultAppbaseGatewayHttpUrl}";
const APPBASE_APP_API_BASE_URL_ENV = "${envPrefix}_APPBASE_APP_API_BASE_URL";
const PLATFORM_API_GATEWAY_HTTP_URL_ENV = "VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL";
const LIFECYCLE_ENVIRONMENTS = new Set<AgentsLifecycleEnvironment>([
  "development",
  "test",
  "staging",
  "production",
]);

function readEnv(environment: AgentsRuntimeEnvironment, key: string): string | undefined {
  const value = environment[key];
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function stripTrailingSlashes(value: string): string {
  let normalized = value.trim();
  while (normalized.endsWith("/")) {
    normalized = normalized.slice(0, -1);
  }
  return normalized;
}

function resolveLifecycleEnvironment(
  environment: AgentsRuntimeEnvironment,
): AgentsLifecycleEnvironment {
  const configuredEnvironment = readEnv(environment, "${envPrefix}_ENVIRONMENT")
    ?? readEnv(environment, "MODE");
  if (configuredEnvironment) {
    const normalized = configuredEnvironment.toLowerCase();
    if (LIFECYCLE_ENVIRONMENTS.has(normalized as AgentsLifecycleEnvironment)) {
      return normalized as AgentsLifecycleEnvironment;
    }
    throw new Error(
      "${envPrefix}_ENVIRONMENT must be development, test, staging, or production.",
    );
  }

  return environment.PROD === true || environment.PROD === "true"
    ? "production"
    : "development";
}

export function normalizeAppbaseGatewayBaseUrl(value: string): string {
  const normalized = stripTrailingSlashes(value);
  if (!normalized) {
    throw new Error("Appbase IAM gateway URL must not be empty.");
  }

  let parsed: URL;
  try {
    parsed = new URL(normalized);
  } catch {
    throw new Error("Appbase IAM gateway URL must be an absolute HTTP(S) URL.");
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    throw new Error("Appbase IAM gateway URL must use HTTP or HTTPS.");
  }
  if (parsed.search || parsed.hash) {
    throw new Error("Appbase IAM gateway URL must not include a query string or fragment.");
  }

  const duplicatedAppApiSuffix = APP_API_SUFFIX + APP_API_SUFFIX;
  if (normalized.endsWith(duplicatedAppApiSuffix)) {
    throw new Error("Appbase IAM gateway URL must not include " + APP_API_SUFFIX + " more than once.");
  }

  return normalized.endsWith(APP_API_SUFFIX)
    ? normalized.slice(0, -APP_API_SUFFIX.length)
    : normalized;
}

function resolveAppbaseGatewayBaseUrl(
  environment: AgentsRuntimeEnvironment,
  lifecycleEnvironment: AgentsLifecycleEnvironment,
): string {
  const configuredBaseUrl = readEnv(environment, APPBASE_APP_API_BASE_URL_ENV)
    ?? readEnv(environment, PLATFORM_API_GATEWAY_HTTP_URL_ENV);
  if (configuredBaseUrl) {
    return normalizeAppbaseGatewayBaseUrl(configuredBaseUrl);
  }
  if (lifecycleEnvironment === "development") {
    return DEVELOPMENT_APPBASE_GATEWAY_HTTP_URL;
  }

  throw new Error(
    APPBASE_APP_API_BASE_URL_ENV
      + " or "
      + PLATFORM_API_GATEWAY_HTTP_URL_ENV
      + " is required for "
      + lifecycleEnvironment
      + ".",
  );
}

function resolveDefaultApplicationPublicHttpUrl(): string {
  return typeof window === "undefined"
    ? DEVELOPMENT_APPLICATION_PUBLIC_HTTP_URL
    : window.location.origin;
}

function deriveAppApiBaseUrl(applicationPublicHttpUrl: string): string {
  return \`\${applicationPublicHttpUrl.replace(/\\/+\$/u, "")}/app/v3/api\`;
}

function deriveBackendApiBaseUrl(applicationPublicHttpUrl: string): string {
  return \`\${applicationPublicHttpUrl.replace(/\\/+\$/u, "")}/backend/v3/api\`;
}

export function createAgentsEnvironment(
  environment: AgentsRuntimeEnvironment,
): AgentsEnvironment {
  const lifecycleEnvironment = resolveLifecycleEnvironment(environment);
  const applicationPublicHttpUrl = readEnv(
    environment,
    "${envPrefix}_APPLICATION_PUBLIC_HTTP_URL",
  ) ?? resolveDefaultApplicationPublicHttpUrl();
  const appbaseAppApiBaseUrl = resolveAppbaseGatewayBaseUrl(environment, lifecycleEnvironment);

  return {
    apiBaseUrl: readEnv(environment, "${envPrefix}_APP_API_BASE_URL")
      ?? deriveAppApiBaseUrl(applicationPublicHttpUrl),
    appbaseAppApiBaseUrl,
    backendApiBaseUrl: readEnv(environment, "${envPrefix}_BACKEND_API_BASE_URL")
      ?? deriveBackendApiBaseUrl(applicationPublicHttpUrl),
    appbaseLoginUrl: readEnv(environment, "${envPrefix}_APPBASE_LOGIN_URL")
      ?? appbaseAppApiBaseUrl,
    lifecycleEnvironment,
  };
}

export function resolveEnvironment(): AgentsEnvironment {
  return createAgentsEnvironment(import.meta.env as AgentsRuntimeEnvironment);
}
`;

  write(path.join(appRoot, 'src/bootstrap/environment.ts'), environmentTs);
  write(path.join(appRoot, 'src/bootstrap/runtime.ts'), `import { registerHostAdapters } from "./hostAdapters";
import { createRoutes } from "./routes";
import { bootstrapSdkClients } from "./sdkClients";

export function bootstrap() {
  registerHostAdapters();
  bootstrapSdkClients();
  createRoutes();
}
`);
  write(path.join(appRoot, 'src/bootstrap/sdkClients.ts'), `import { resolveEnvironment } from "./environment";

export function bootstrapSdkClients() {
  const environment = resolveEnvironment();
  return {
    apiBaseUrl: environment.apiBaseUrl,
    backendApiBaseUrl: environment.backendApiBaseUrl,
  };
}
`);
  write(path.join(appRoot, 'src/bootstrap/hostAdapters.ts'), `export function registerHostAdapters() {
  return {};
}
`);
  write(path.join(appRoot, 'src/bootstrap/routes.ts'), `export function createRoutes() {
  return [];
}
`);
  write(path.join(appRoot, 'src/bootstrap/iamRuntime.ts'), `export function createIamRuntime() {
  return null;
}
`);

  createTsPackage(appRoot, `sdkwork-${applicationCode}-${packageSegment}-core`, `@sdkwork/${applicationCode}-${packageSegment}-core`, 'core');
  createTsPackage(appRoot, `sdkwork-${applicationCode}-${packageSegment}-commons`, `@sdkwork/${applicationCode}-${packageSegment}-commons`, 'commons');
  createTsPackage(appRoot, `sdkwork-${applicationCode}-${packageSegment}-shell`, `@sdkwork/${applicationCode}-${packageSegment}-shell`, 'shell');
  for (const pkg of extraPackages) {
    createTsPackage(appRoot, pkg.dir, pkg.name, pkg.capability);
  }

  return appRootName;
}

function createMiniProgramSurface() {
  const suffix = 'mini-program';
  const appRootName = `sdkwork-${applicationCode}-${suffix}`;
  const appRoot = path.join(repoRoot, 'apps', appRootName);
  const manifestKey = `${applicationCode}-${suffix}`;
  const appId = backendAppId;

  write(path.join(appRoot, 'AGENTS.md'), agentsMd(
    'Mini Program',
    'MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md',
    appId,
    '```powershell\npnpm install\npnpm run build\npnpm run typecheck\n```',
  ));
  writeJson(path.join(appRoot, 'agent.app.config.json'), manifestV3({
    key: manifestKey,
    displayName: 'SDKWork Agents Mini Program',
    description: 'SDKWork Agents WeChat mini program client application scaffold.',
    appType: 'APP_UNIAPP',
    appId,
    platform: 'MP_WEIXIN',
    framework: 'mp-weixin',
    family: 'mini-program',
    runtimes: ['MP_WEIXIN'],
    defaultPlatform: 'MP_WEIXIN',
    sourceRoot: `apps/${appRootName}`,
  }));
  writeReadme(path.join(appRoot, '.sdkwork/README.md'), sdkworkDotSdkworkReadme());
  writeReadme(path.join(appRoot, '.sdkwork/skills/README.md'), '# Skills\n');
  writeReadme(path.join(appRoot, '.sdkwork/plugins/README.md'), '# Plugins\n');

  writeJson(path.join(appRoot, 'specs/component.spec.json'), {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: appRootName,
      displayName: 'SDKWork Agents Mini Program',
      version: '0.1.0',
      type: 'mini-program-app-root',
      root: `apps/${appRootName}`,
      domain: applicationCode,
      capability: applicationCode,
      surface: 'app',
      languages: ['typescript'],
      generated: false,
      manifests: ['package.json', 'agent.app.config.json', 'project.config.json'],
    },
    contracts: {
      sdkDependencies: [
        { workspace: 'sdkwork-agents-app-sdk', surface: 'app-api', credentialMode: 'authenticated-app-api' },
      ],
    },
    verification: {
      commands: ['pnpm run build', 'pnpm run typecheck'],
    },
  });

  writeJson(path.join(appRoot, 'config/mini-program/runtime-env.development.example.json'), {
    agents: {
      apiBaseUrl: `${defaultPublicHttpUrl}/app/v3/api`,
    },
    appbase: {
      loginUrl: defaultAppbaseGatewayHttpUrl,
    },
  });
  writeJson(path.join(appRoot, 'config/host/mp-weixin.development.example.json'), {
    appId: 'touristappid',
    projectName: appRootName,
  });

  writeJson(path.join(appRoot, 'project.config.json'), {
    miniprogramRoot: 'src/',
    projectname: appRootName,
    appid: 'touristappid',
    setting: { urlCheck: false, es6: true, minified: true },
    compileType: 'miniprogram',
  });

  writeJson(path.join(appRoot, 'package.json'), {
    name: `@sdkwork/${applicationCode}-mini-program`,
    private: true,
    version: '0.1.0',
    type: 'module',
    scripts: {
      build: 'node scripts/build-runtime.mjs',
      typecheck: 'tsc --noEmit -p tsconfig.json',
    },
    dependencies: {
      '@sdkwork/agents-mp-core': 'workspace:*',
      '@sdkwork/agents-mp-host': 'workspace:*',
      '@sdkwork/sdk-common': 'workspace:*',
    },
    devDependencies: {
      esbuild: '^0.25.10',
      'miniprogram-api-typings': '^4.1.0',
      typescript: 'catalog:',
    },
  });

  write(path.join(appRoot, 'pnpm-workspace.yaml'), 'packages:\n  - "packages/*"\n');

  writeJson(path.join(appRoot, 'tsconfig.json'), {
    compilerOptions: {
      target: 'ES2019',
      module: 'ESNext',
      moduleResolution: 'bundler',
      strict: true,
      skipLibCheck: true,
      types: ['miniprogram-api-typings'],
      noEmit: true,
    },
    include: ['src', 'packages'],
  });

  write(path.join(appRoot, 'scripts/build-runtime.mjs'), `import * as esbuild from "esbuild";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));

await esbuild.build({
  entryPoints: [path.join(root, "../src/bootstrap/runtimeBundle.ts")],
  bundle: true,
  outfile: path.join(root, "../src/runtime/agents-app.js"),
  platform: "browser",
  format: "cjs",
  target: "es2019",
  logLevel: "info",
});
`);

  write(path.join(appRoot, 'src/app.js'), `const { bootstrapAgentsMiniProgram } = require("./runtime/agents-app");

App({
  onLaunch() {
    try {
      bootstrapAgentsMiniProgram();
    } catch {
      // Runtime bundle is produced by pnpm run build.
    }
    wx.reLaunch({ url: "/pages/home/index" });
  },
});
`);

  writeJson(path.join(appRoot, 'src/app.json'), {
    pages: ['pages/home/index'],
    window: {
      navigationBarTitleText: 'SDKWork Agents',
      navigationBarBackgroundColor: '#0f766e',
      navigationBarTextStyle: 'white',
    },
  });
  write(path.join(appRoot, 'src/app.wxss'), 'page { background: #f8fafc; }\n');
  write(path.join(appRoot, 'src/pages/home/index.js'), `Page({
  data: { title: "SDKWork Agents" },
});
`);
  writeJson(path.join(appRoot, 'src/pages/home/index.json'), { usingComponents: {} });
  write(path.join(appRoot, 'src/pages/home/index.wxml'), '<view class="container"><text>{{title}}</text></view>\n');
  write(path.join(appRoot, 'src/pages/home/index.wxss'), '.container { padding: 32rpx; }\n');

  write(path.join(appRoot, 'src/bootstrap/runtimeBundle.ts'), `import { bootstrap } from "./runtime";

export function bootstrapAgentsMiniProgram() {
  bootstrap();
}
`);
  write(path.join(appRoot, 'src/bootstrap/runtime.ts'), `export function bootstrap() {
  return { ready: true };
}
`);
  write(path.join(appRoot, 'src/bootstrap/environment.ts'), 'export function resolveEnvironment() { return {}; }\n');
  write(path.join(appRoot, 'src/bootstrap/sdkClients.ts'), 'export function bootstrapSdkClients() { return {}; }\n');
  write(path.join(appRoot, 'src/bootstrap/hostAdapters.ts'), 'export function registerHostAdapters() { return {}; }\n');
  write(path.join(appRoot, 'src/bootstrap/routes.ts'), 'export function createRoutes() { return []; }\n');
  write(path.join(appRoot, 'src/bootstrap/iamRuntime.ts'), 'export function createIamRuntime() { return null; }\n');
  write(path.join(appRoot, 'src/runtime/.gitkeep'), '');

  createTsPackage(appRoot, 'sdkwork-agents-mp-core', '@sdkwork/agents-mp-core', 'core');
  createTsPackage(appRoot, 'sdkwork-agents-mp-host', '@sdkwork/agents-mp-host', 'host');

  return appRootName;
}

function createFlutterSurface() {
  const appRootName = `sdkwork-${applicationCode}-flutter-mobile`;
  const appRoot = path.join(repoRoot, 'apps', appRootName);
  const manifestKey = `${applicationCode}-flutter-mobile`;
  const appId = backendAppId;

  write(path.join(appRoot, 'AGENTS.md'), agentsMd(
    'Flutter Mobile',
    'FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md',
    appId,
    '```powershell\nflutter pub get\nflutter analyze\n```',
  ));
  writeJson(path.join(appRoot, 'agent.app.config.json'), manifestV3({
    key: manifestKey,
    displayName: 'SDKWork Agents Mobile',
    description: 'SDKWork Agents Flutter mobile client application scaffold.',
    appType: 'APP_FLUTTER',
    appId,
    platform: 'APP',
    framework: 'flutter',
    family: 'mobile',
    runtimes: ['APP', 'APP_ANDROID', 'APP_IOS'],
    defaultPlatform: 'APP_ANDROID',
    sourceRoot: `apps/${appRootName}`,
    packageName: 'com.sdkwork.agents.mobile',
    bundleId: 'com.sdkwork.agents.mobile',
  }));
  writeReadme(path.join(appRoot, '.sdkwork/README.md'), sdkworkDotSdkworkReadme());
  writeReadme(path.join(appRoot, '.sdkwork/skills/README.md'), '# Skills\n');
  writeReadme(path.join(appRoot, '.sdkwork/plugins/README.md'), '# Plugins\n');

  writeJson(path.join(appRoot, 'specs/component.spec.json'), {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: appRootName,
      displayName: 'SDKWork Agents Flutter Mobile',
      version: '0.1.0',
      type: 'flutter-mobile-app-root',
      root: `apps/${appRootName}`,
      domain: applicationCode,
      capability: applicationCode,
      surface: 'app',
      languages: ['dart'],
      generated: false,
      manifests: ['pubspec.yaml', 'agent.app.config.json'],
    },
    contracts: {
      sdkDependencies: [
        { workspace: 'sdkwork-agents-app-sdk', surface: 'app-api', credentialMode: 'authenticated-app-api' },
      ],
    },
    verification: {
      commands: ['flutter analyze'],
    },
  });

  writeJson(path.join(appRoot, 'config/app/runtime-env.development.example.json'), {
    agents: {
      apiBaseUrl: `${defaultPublicHttpUrl}/app/v3/api`,
    },
    appbase: {
      loginUrl: defaultAppbaseGatewayHttpUrl,
    },
  });
  write(path.join(appRoot, '.env.example'), 'SDKWORK_ACCESS_TOKEN=\n');

  write(path.join(appRoot, 'pubspec.yaml'), `name: sdkwork_agents_flutter_mobile
description: SDKWork Agents Flutter Mobile Application
version: 0.1.0
publish_to: none

environment:
  sdk: ">=3.5.0 <4.0.0"
  flutter: ">=3.24.0"

dependencies:
  flutter:
    sdk: flutter
  sdkwork_agents_flutter_mobile_core:
    path: packages/sdkwork_agents_flutter_mobile_core

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^5.0.0

flutter:
  uses-material-design: true
`);

  write(path.join(appRoot, 'lib/main.dart'), `import 'package:flutter/material.dart';

import 'app.dart';
import 'bootstrap/runtime.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await bootstrap();
  runApp(const AgentsApp());
}
`);

  write(path.join(appRoot, 'lib/app.dart'), `import 'package:flutter/material.dart';

import 'auth_gate.dart';

class AgentsApp extends StatelessWidget {
  const AgentsApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'SDKWork Agents',
      theme: ThemeData(colorSchemeSeed: const Color(0xFF0F766E)),
      home: const AuthGate(),
    );
  }
}
`);

  write(path.join(appRoot, 'lib/auth_gate.dart'), `import 'package:flutter/material.dart';

class AuthGate extends StatelessWidget {
  const AuthGate({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(
        child: Text('SDKWork Agents Flutter scaffold'),
      ),
    );
  }
}
`);

  write(path.join(appRoot, 'lib/bootstrap/runtime.dart'), `import 'host_adapters.dart';
import 'iam_runtime.dart';
import 'routes.dart';
import 'sdk_clients.dart';

Future<void> bootstrap() async {
  createIamRuntime();
  registerHostAdapters();
  createSdkClients();
  createRoutes();
}
`);
  write(path.join(appRoot, 'lib/bootstrap/iam_runtime.dart'), 'void createIamRuntime() {}\n');
  write(path.join(appRoot, 'lib/bootstrap/sdk_clients.dart'), 'void createSdkClients() {}\n');
  write(path.join(appRoot, 'lib/bootstrap/host_adapters.dart'), 'void registerHostAdapters() {}\n');
  write(path.join(appRoot, 'lib/bootstrap/routes.dart'), 'void createRoutes() {}\n');

  const coreDir = path.join(appRoot, 'packages/sdkwork_agents_flutter_mobile_core');
  write(path.join(coreDir, 'pubspec.yaml'), `name: sdkwork_agents_flutter_mobile_core
description: SDKWork Agents Flutter mobile core package
version: 0.1.0
publish_to: none

environment:
  sdk: ">=3.5.0 <4.0.0"

dependencies:
  flutter:
    sdk: flutter
`);
  write(path.join(coreDir, 'lib/sdkwork_agents_flutter_mobile_core.dart'), 'library sdkwork_agents_flutter_mobile_core;\n');
  writeJson(path.join(coreDir, 'specs/component.spec.json'), {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: 'sdkwork_agents_flutter_mobile_core',
      type: 'dart-package',
      domain: applicationCode,
      capability: 'core',
      surface: 'app',
      languages: ['dart'],
      generated: false,
    },
    contracts: {},
  });

  return appRootName;
}

function createRootWorkspace() {
  write(path.join(repoRoot, 'pnpm-workspace.yaml'), `packages:
  - "apps/*"
  - "apps/*/packages/*"
  - "../sdkwork-sdk-commons/sdkwork-sdk-common-typescript"
  - "sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript"

catalog:
  "@tailwindcss/vite": ^4.1.14
  "@types/react": ^19.2.14
  "@types/react-dom": ^19.2.3
  "@vitejs/plugin-react": ^6.0.1
  "i18next": ^26.1.0
  "jsdom": ^29.1.1
  "lucide-react": ^1.7.0
  "qrcode": ^1.5.4
  "react": ^19.2.4
  "react-dom": ^19.2.4
  "react-hook-form": ^7.72.1
  "react-i18next": ^17.0.7
  "typescript": ^6.0.2
  "tailwindcss": ^4.1.14
  "vite": ^8.0.3
  "vite-plugin-dts": ^4.5.4
  "vitest": ^4.1.8
`);
}

function updateAppsReadme(appRoots) {
  const content = `# Application Surfaces

The repository root is the primary API/server application surface for
\`sdkwork-agents\`. Client application roots live under \`apps/\` and follow
\`APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md\`.

## Directory Index

| Application root | Architecture | Runnable | Standard |
| --- | --- | --- | --- |
${appRoots.map((name) => {
  const arch = name.includes('flutter') ? 'flutter-mobile' : name.includes('mini-program') ? 'mini-program' : name.endsWith('-h5') ? 'h5' : 'pc';
  const spec = {
    pc: 'APP_PC_ARCHITECTURE_SPEC.md',
    h5: 'APP_H5_ARCHITECTURE_SPEC.md',
    'mini-program': 'MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md',
    'flutter-mobile': 'FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md',
  }[arch];
  return `| [\`${name}/\`](./${name}/) | ${arch} | yes | [\`${spec}\`](../../sdkwork-specs/${spec}) |`;
}).join('\n')}

## References

- [\`APPLICATION_SPEC.md\`](../../sdkwork-specs/APPLICATION_SPEC.md)
- [\`SDKWORK_WORKSPACE_SPEC.md\`](../../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md)
- [\`APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md\`](../../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md)
`;
  fs.writeFileSync(path.join(repoRoot, 'apps/README.md'), content);
}

const created = [];
created.push(createReactSurface({
  suffix: 'pc',
  archId: 'pc',
  archSpec: 'APP_PC_ARCHITECTURE_SPEC.md',
  appType: 'APP_REACT',
  platform: 'WEB',
  framework: 'react',
  family: 'web',
  runtimes: ['WEB'],
  defaultPlatform: 'WEB',
  vitePort: 5195,
  envPrefix: 'VITE_SDKWORK_AGENTS_PC',
  packageSegment: 'pc',
}));
created.push(createReactSurface({
  suffix: 'h5',
  archId: 'h5',
  archSpec: 'APP_H5_ARCHITECTURE_SPEC.md',
  appType: 'APP_REACT',
  platform: 'H5',
  framework: 'react-h5',
  family: 'mobile',
  runtimes: ['H5'],
  defaultPlatform: 'H5',
  vitePort: 5196,
  envPrefix: 'VITE_SDKWORK_AGENTS_H5',
  packageSegment: 'h5',
}));
created.push(createMiniProgramSurface());
created.push(createFlutterSurface());
createRootWorkspace();
updateAppsReadme(created);

console.log(`materialize-client-app-surfaces: initialized ${created.length} client roots`);
for (const name of created) {
  console.log(`- apps/${name}`);
}
