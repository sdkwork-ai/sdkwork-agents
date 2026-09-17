#!/usr/bin/env node

// Materializes the SDKWork Agents HarmonyOS native mobile application root
// (`apps/sdkwork-agents-harmony-mobile`) following
// `HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md` and
// `APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md`.
//
// Design rules:
// - Idempotent: existing files are never overwritten.
// - Scope: only touches `apps/sdkwork-agents-harmony-mobile`.
// - Never writes `apps/README.md` (owned by sdkwork-specs align tooling).
// - Never writes a nested app-level `pnpm-workspace.yaml` (forbidden by
//   verify-repo). Harmony roots are ohpm/hvigor workspaces, not pnpm members.
// - Never declares an SDK dependency that does not exist in this workspace.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const applicationCode = 'agents';
const appRootName = `sdkwork-${applicationCode}-harmony-mobile`;
const appRoot = path.join(repoRoot, 'apps', appRootName);
const backendAppId = 'sdkwork-agents';
const bundleName = 'com.sdkwork.agents.mobile';
const runtimeTarget = 'harmony-native';
const publicHttpUrl = 'http://127.0.0.1:8095';

const deploymentProfiles = ['standalone', 'cloud'];
const environments = ['development', 'test', 'staging', 'production'];

const created = [];
const skipped = [];

function write(relativePath, content) {
  const absolute = path.join(appRoot, relativePath);
  fs.mkdirSync(path.dirname(absolute), { recursive: true });
  if (fs.existsSync(absolute)) {
    skipped.push(relativePath);
    return false;
  }
  fs.writeFileSync(absolute, content);
  created.push(relativePath);
  return true;
}

function writeJson(relativePath, value) {
  return write(relativePath, `${JSON.stringify(value, null, 2)}\n`);
}

// ---------------------------------------------------------------------------
// Root metadata
// ---------------------------------------------------------------------------

/**
 * Application-root AGENTS.md.
 *
 * Section set and spec references are fixed by
 * `sdkwork-specs/tools/check-agent-workflow-standard.mjs`
 * (REQUIRED_AGENT_SECTIONS + SPEC_REFERENCES). Relative depth matches the
 * sibling roots under `apps/`: specs are three levels up, the repository
 * dictionary two levels up.
 */
function agentsMd() {
  return `# Repository Guidelines

## SDKWORK Soul

Read \`../../../sdkwork-specs/SOUL.md\` before executing application tasks. Start with the sections that route the current task; related-spec references are not a startup bundle.

## SDKWORK Standards

The canonical standards index is \`../../../sdkwork-specs/README.md\`, and \`../../../sdkwork-specs/AGENTS_SPEC.md\` governs this entrypoint. Read the relevant task-matrix row first and do not copy global normative bodies locally.

## Application Identity

Read \`sdkwork.app.config.json\` only for application identity, SDK/API inventory, release metadata, packaging, or app-owned capabilities. Runtime values belong to source configuration under \`etc/\` and \`config/\`, not to the application declaration.

- Application code: \`${applicationCode}\`
- Application key: \`${appRootName}\`
- Bundle name: \`${bundleName}\`
- Client architecture: \`harmony-mobile\` (runtime target \`${runtimeTarget}\`)

## Local Dictionary Structure

Use \`AGENTS.md\` as the application routing entrypoint. Read \`.sdkwork/\`, \`specs/\`, application source, tests, and documentation only when the current task reaches the contract each location governs.

- \`entry/\` is the installable HarmonyOS entry/composition module: the entry ability, bootstrap, provider assembly, route registry, SDK client construction, IAM runtime wiring, and host adapter registration.
- \`packages/\` owns ArkTS/HAR reusable core, commons, shell, host, and capability packages.
- \`config/\`, \`etc/\`, \`AppScope/\`, \`oh-package.json5\`, \`build-profile.json5\`, and \`hvigor/\` are deployable-root configuration and build metadata.

## Spec Resolution Order

Use dynamic progressive loading: read this file and \`../../AGENTS.md\`, then \`../../../sdkwork-specs/HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\`, then applicable local contracts under \`specs/\`, then the relevant task route in \`../../../sdkwork-specs/README.md\`, and only afterward inspect implementation files. Language-specific specs are on-demand only.

## Required Specs By Task Type

HarmonyOS client work loads \`../../../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md\`, \`../../../sdkwork-specs/HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\`, and \`../../../sdkwork-specs/APP_HARMONY_NATIVE_UI_SPEC.md\`. Code changes load \`../../../sdkwork-specs/CODE_STYLE_SPEC.md\`, \`../../../sdkwork-specs/NAMING_SPEC.md\`, and only the touched frontend authority such as \`../../../sdkwork-specs/FRONTEND_CODE_SPEC.md\`. Package-command work loads \`../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md\`; packaging workflow work loads \`../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md\`; deployment work loads \`../../../sdkwork-specs/DEPLOYMENT_SPEC.md\` and \`../../../sdkwork-specs/CONFIG_SPEC.md\`.

## Code Style Rules

Consume remote capabilities through the generated ArkTS/TypeScript app SDK clients composed in \`core\`/bootstrap. Do not introduce raw HTTP, manual authentication headers, generated transport imports, local SDK forks, duplicated shared utilities, or a second appbase IAM runtime. Feature packages must not construct SDK clients or read runtime environment values directly.

## Build, Test, and Verification

HarmonyOS builds require DevEco Studio or compatible HarmonyOS SDK, \`hvigor\`, and \`ohpm\` tooling plus a documented signing profile; these toolchains are not part of the repository workspace and are tracked as pending integration.

\`\`\`powershell
ohpm install
hvigor clean
hvigor assembleHap
\`\`\`

Static repository verification (runs without the HarmonyOS toolchain):

\`\`\`bash
node ../../sdkwork-specs/tools/check-apps-directory-index.mjs --root ../..
node ../../sdkwork-specs/tools/check-frontend-composition.mjs --root ../..
node ../../sdkwork-specs/tools/check-component-port-bindings.mjs --root ../..
node --test tests/harmony-surface-contract.test.mjs
\`\`\`

## Agent Execution Rules

Follow specifications before memory and evidence before completion. Keep SDK construction, authentication, environment selection, and host capabilities in their owning composition layers. Stop when kernel ownership, API authority, or SDK family boundaries are ambiguous.

## Task-Specific Standards

SDK consumer work loads \`../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md\` and runs \`check-app-sdk-consumer-imports.mjs\`. API work loads \`../../../sdkwork-specs/API_SPEC.md\` and its validators. List/search work loads \`../../../sdkwork-specs/PAGINATION_SPEC.md\` and \`check-pagination.mjs\`. Source configuration work loads \`../../../sdkwork-specs/SOURCE_CONFIG_SPEC.md\` and \`check-source-config-standard.mjs\`. Locale resource work loads \`../../../sdkwork-specs/I18N_SPEC.md\`.

## Human Review Rules

Human review is required for public API changes, security exceptions, database migrations, generated SDK ownership changes, destructive operations, and cross-application standards changes.
`;
}

function readmeMd() {
  return `# SDKWork Agents HarmonyOS Mobile

Native HarmonyOS (ArkTS/ArkUI) client application root for SDKWork Agents.

## Status

Architecture scaffold materialized against
\`HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\`. This is the first
\`-harmony-mobile\` application root in the SDKWork workspace, so the package
family, composition contracts, and host adapter boundaries are established
here for the first time.

## Blocking Prerequisites

The following are **not yet satisfied** and are required before this root can
produce a signed HAP:

1. **HarmonyOS toolchain.** \`ohpm\`, \`hvigor\`, and the HarmonyOS SDK are not
   installed in the current development environment. \`hvigor assembleHap\`
   and \`ohpm install\` cannot run yet.
2. **ArkTS SDK adaptation.** \`HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 6
   requires Harmony packages to consume \`/app/v3/api\` through generated
   ArkTS/TypeScript app SDK clients *adapted for the Harmony runtime*. This
   repository generates the TypeScript (\`sdkwork-agents-app-sdk-typescript\`)
   and Flutter (\`sdkwork-agents-app-sdk-flutter\`) targets of
   \`sdkwork-agents-app-sdk\`; **no ArkTS target is produced by the SDK
   generation chain yet**. \`core\` therefore declares the SDK port contract and
   the token/configuration boundary, and does not fabricate a vendored copy of
   the transport.
3. **Bundle signing profile.** \`config/host/harmony.*.example.json\` are
   secret-free templates; a real signing profile reference must be supplied by
   DevEco Studio or CI secure storage.

## Package Family

| Package | Role | Layer role |
| --- | --- | --- |
| \`packages/sdkwork-agents-harmony-mobile-core\` | runtime config, SDK factories, token manager, session stores, route registry, host adapter contracts | frontend-core |
| \`packages/sdkwork-agents-harmony-mobile-commons\` | domain-neutral ArkUI components, theme adapters, i18n helpers | frontend-commons |
| \`packages/sdkwork-agents-harmony-mobile-shell\` | app shell, navigation/page stack, AuthGate integration | frontend-shell |
| \`packages/sdkwork-agents-harmony-mobile-host\` | typed HarmonyOS host adapters (camera, QR, secure storage, push, ...) | frontend-host |
| \`packages/sdkwork-agents-harmony-mobile-agents\` | agent catalog, creation, and conversation capability | frontend-feature |

## Configuration

Non-secret runtime config materializes as
\`config/app/runtime-env.<deploymentProfile>.<environment>.json\` and declares
matching \`environment\`, \`deploymentProfile\`, \`profileId\`, and
\`runtimeTarget=${runtimeTarget}\`. Host/platform metadata belongs to
\`config/host/\` and must stay secret-free.

## Verification

Static verification runs today, without the HarmonyOS toolchain (from this
directory):

\`\`\`bash
node ../../sdkwork-specs/tools/check-apps-directory-index.mjs --root ../..
node ../../sdkwork-specs/tools/check-frontend-composition.mjs --root ../..
node ../../sdkwork-specs/tools/check-component-port-bindings.mjs --root ../..
node --test tests/harmony-surface-contract.test.mjs
\`\`\`

Repository-wide gates that also cover this root (from the repository root,
requires \`pnpm install\`):

\`\`\`bash
pnpm check
\`\`\`

There is deliberately no \`check:harmony-native\` script: no HarmonyOS build
command can run until prerequisite 1 is satisfied, and a script that cannot
execute would be a false signal.
`;
}

function appManifest() {
  return {
    schemaVersion: 3,
    kind: 'sdkwork.app',
    app: {
      key: `${applicationCode}-harmony-mobile`,
      name: 'SDKWork Agents HarmonyOS Mobile',
      displayName: 'SDKWork Agents HarmonyOS Mobile',
      description: 'SDKWork Agents native HarmonyOS (ArkTS/ArkUI) mobile client application.',
      vendor: 'SDKWork',
      officialWebsiteUrl: 'https://sdkwork.com/apps/agents-harmony-mobile',
      supportUrl: 'https://sdkwork.com/support',
      privacyPolicyUrl: 'https://sdkwork.com/privacy',
      termsOfServiceUrl: 'https://sdkwork.com/terms',
      appType: 'APP_HARMONY',
      versionSource: 'oh-package.json5',
      identifiers: {
        packageName: bundleName,
        bundleId: bundleName,
        desktopAppId: null,
        containerImage: 'registry.sdkwork.com/apps/agents-harmony-mobile',
      },
    },
    backend: {
      profileKey: 'backend-root-admin',
      ownerMode: 'tenant',
      grantMode: 'current',
      platform: 'APP_HARMONY',
      appId: backendAppId,
      organizationId: '0',
      tenantId: '100001',
      accessTokenPermissionScope: [
        'iam.users.read',
        'iam.organizations.read',
        'iam.roles.read',
        'iam.permissions.read',
      ],
    },
    runtime: {
      family: 'mobile',
      framework: 'harmony-native',
      runtimes: ['APP_HARMONY'],
      deliveryModes: ['APP_GALLERY', 'DIRECT_DOWNLOAD'],
      defaultPlatform: 'APP_HARMONY',
      defaultArchitecture: runtimeTarget,
      supportedDeploymentProfiles: deploymentProfiles,
      defaultDeploymentProfile: 'standalone',
    },
    media: {
      icons: {
        primary: {
          id: 'agents-harmony-mobile-primary-icon',
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
      status: 'DRAFT',
      installSkill: false,
      platforms: ['APP_HARMONY'],
      installPlatforms: ['APP_HARMONY'],
      config: {
        workspaceRoot: `apps/${appRootName}`,
        framework: 'harmony-native',
        managedBy: 'sdkwork-agents-harmony-scaffold',
        distribution: {
          appGallery: {
            appIdPlaceholder: '<appgallery-app-id>',
            signingReference: '<deveco-signing-profile-name>',
          },
          privateDistribution: {
            channelPlaceholder: '<private-distribution-channel>',
          },
        },
      },
    },
    artifacts: {
      installConfig: {
        packages: [],
        metadata: {
          workspaceRoot: `apps/${appRootName}`,
          framework: 'harmony-native',
          packageManager: 'ohpm',
          deferred: true,
          deferredReason: 'HarmonyOS SDK/hvigor/ohpm toolchain and signing profile are not available in the current environment.',
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
      signatureRequired: true,
      sbomRequired: true,
    },
    devApp: {
      build: { targets: [] },
      sourceRoot: `apps/${appRootName}`,
    },
    metadata: {
      standardOwner: 'sdkwork-agents',
      architectureSpec: 'HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md',
      runtimeTarget,
    },
  };
}

function rootComponentSpec() {
  return {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: appRootName,
      displayName: 'SDKWork Agents HarmonyOS Mobile',
      version: '0.1.0',
      type: 'harmony-mobile-app-root',
      root: `apps/${appRootName}`,
      domain: applicationCode,
      capability: applicationCode,
      surface: 'app',
      languages: ['arkts'],
      generated: false,
      manifests: ['oh-package.json5', 'build-profile.json5', 'sdkwork.app.config.json'],
    },
    canonicalSpecs: [
      {
        file: 'HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md',
        path: '../../../sdkwork-specs/HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md',
        purpose: 'HarmonyOS native mobile application root architecture.',
      },
      {
        file: 'HARMONY_APP_NATIVE_UI_SPEC.md',
        path: '../../../sdkwork-specs/APP_HARMONY_NATIVE_UI_SPEC.md',
        purpose: 'Harmony native ArkUI package rules.',
      },
      {
        file: 'APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md',
        path: '../../../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md',
        purpose: 'Cross-client package taxonomy and dependency direction.',
      },
      {
        file: 'APP_COMPOSITION_SPEC.md',
        path: '../../../sdkwork-specs/APP_COMPOSITION_SPEC.md',
        purpose: 'Native-authority application composition.',
      },
      {
        file: 'APP_SDK_INTEGRATION_SPEC.md',
        path: '../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md',
        purpose: 'Generated ArkTS/TypeScript app SDK and TokenManager integration.',
      },
    ],
    contracts: {
      publicExports: ['entry/src/main/ets/entryability/EntryAbility.ets'],
      runtimeEntrypoints: ['package.json#scripts.dev:harmony-native:standalone'],
      sdkClients: ['SdkworkAppClient'],
      sdkDependencies: ['sdkwork-agents-app-sdk'],
      dependencyApiExports: [],
      dependencyApiSurfaces: [],
    },
    verification: {
      commands: [
        'node ../sdkwork-specs/tools/check-apps-directory-index.mjs --root .',
        'node --test apps/sdkwork-agents-harmony-mobile/tests/harmony-surface-contract.test.mjs',
      ],
    },
  };
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

function runtimeEnv(deploymentProfile, environment) {
  const isStandalone = deploymentProfile === 'standalone';
  return {
    environment,
    deploymentProfile,
    profileId: `${deploymentProfile}.${environment}`,
    runtimeTarget,
    agents: {
      apiBaseUrl: `${publicHttpUrl}/app/v3/api`,
      backendApiBaseUrl: `${publicHttpUrl}/backend/v3/api`,
    },
    appbase: {
      appApiBaseUrl: publicHttpUrl,
      loginUrl: publicHttpUrl,
    },
    metadata: {
      profileBinding: isStandalone ? 'runtime-configurable' : 'runtime-configurable',
      applicationPublicHttpUrl: publicHttpUrl,
    },
  };
}

function harmonyHostExample(environment) {
  return {
    environment,
    bundleName,
    vendor: 'SDKWork',
    deviceTypes: ['phone', 'tablet'],
    permissions: [
      'ohos.permission.INTERNET',
      'ohos.permission.GET_NETWORK_INFO',
    ],
    signingReference: '<deveco-signing-profile-name>',
    distribution: {
      appGallery: { appIdPlaceholder: '<appgallery-app-id>' },
      privateDistribution: { channelPlaceholder: '<private-distribution-channel>' },
    },
    wants: [
      {
        scheme: 'sdkwork-agents',
        host: 'app',
        pathPrefix: '/app/agents',
      },
    ],
  };
}

function materializeConfig() {
  for (const deploymentProfile of deploymentProfiles) {
    for (const environment of environments) {
      writeJson(
        `config/app/runtime-env.${deploymentProfile}.${environment}.json`,
        runtimeEnv(deploymentProfile, environment),
      );
    }
  }
  writeJson('config/app/runtime-env.development.example.json', {
    ...runtimeEnv('standalone', 'development'),
    deploymentProfile: 'standalone',
    profileId: 'standalone.development',
    metadata: {
      purpose: 'Checked-in safe template. Copy to a concrete profile file to edit.',
    },
  });

  for (const environment of environments) {
    writeJson(`config/host/harmony.${environment}.example.json`, harmonyHostExample(environment));
  }

  write('config/host/README.md', `# Host Config

HarmonyOS bundle id, module metadata, device types, permissions, app links,
push profile references, signing reference names, and AppGallery/private
distribution references belong here.

Rules:

- Safe checked-in templates only. Files use the \`.example.json\` suffix.
- Must not contain signing private keys, auth tokens, refresh tokens, API keys,
  database credentials, private service endpoints, SDK ownership, or business
  route constants.
`);

  write('config/server/agents.development.toml.example', `# Server profile template for the SDKWork Agents HarmonyOS mobile surface.
# Shared deployment authority lives at ../../../etc/sdkwork.deployment.config.json.
[runtime]
target = "${runtimeTarget}"
profile = "development"
`);

  write('config/container/agents.development.toml.example', `# Container profile template for the SDKWork Agents HarmonyOS mobile surface.
[runtime]
target = "${runtimeTarget}"
profile = "development"
`);
}

function materializeEtc() {
  write('etc/README.md', `# Component Deployment

This application surface shares the enclosing application deployment unit.
Deployment profiles are owned by \`../../../etc/sdkwork.deployment.config.json\`;
runtime process topology is owned by \`../../../specs/topology.spec.json\`.
Surface-local build and test commands stay in this application root.
`);

  const profiles = {};
  for (const deploymentProfile of deploymentProfiles) {
    for (const environment of environments) {
      const profileId = `${deploymentProfile}.${environment}`;
      profiles[profileId] = { source: `../config/app/runtime-env.${profileId}.json` };
    }
  }

  writeJson('etc/sdkwork.deployment.config.json', {
    schemaVersion: 1,
    kind: 'sdkwork.component-deployment',
    application: `${applicationCode}-harmony-mobile`,
    parentDeploymentConfig: '../../../etc/sdkwork.deployment.config.json',
    parentTopologySpec: '../../../specs/topology.spec.json',
    materialization: {
      authority: '../../../etc/sdkwork.deployment.config.json',
      command: 'pnpm workflow:materialize-client-env',
      format: 'json',
      outputPattern: '../config/app/runtime-env.{deploymentProfile}.{environment}.json',
      profiles: Object.keys(profiles),
      runtimeTarget,
    },
    profiles,
  });
}

// ---------------------------------------------------------------------------
// HarmonyOS workspace files
// ---------------------------------------------------------------------------

function materializeWorkspaceFiles() {
  write('AGENTS.md', agentsMd());
  write('README.md', readmeMd());
  writeJson('sdkwork.app.config.json', appManifest());

  write('.sdkwork/README.md', `# SDKWork Application Workspace

Local skills and plugins for this client application root follow \`SDKWORK_WORKSPACE_SPEC.md\`.
`);
  write('.sdkwork/skills/README.md', `# Skills

Application-local skills for the SDKWork Agents HarmonyOS mobile root.
`);
  write('.sdkwork/plugins/README.md', `# Plugins

Application-local plugins for the SDKWork Agents HarmonyOS mobile root.
`);

  write('oh-package.json5', `{
  "modelVersion": "5.0.0",
  "name": "${appRootName}",
  "version": "0.1.0",
  "description": "SDKWork Agents HarmonyOS native mobile application",
  "main": "",
  "author": "SDKWork",
  "license": "Apache-2.0",
  "dependencies": {
    "@sdkwork/sdkwork-agents-harmony-mobile-core": "file:./packages/sdkwork-agents-harmony-mobile-core",
    "@sdkwork/sdkwork-agents-harmony-mobile-commons": "file:./packages/sdkwork-agents-harmony-mobile-commons",
    "@sdkwork/sdkwork-agents-harmony-mobile-shell": "file:./packages/sdkwork-agents-harmony-mobile-shell",
    "@sdkwork/sdkwork-agents-harmony-mobile-host": "file:./packages/sdkwork-agents-harmony-mobile-host",
    "@sdkwork/sdkwork-agents-harmony-mobile-agents": "file:./packages/sdkwork-agents-harmony-mobile-agents"
  },
  "devDependencies": {
    "@ohos/hypium": "1.0.19"
  }
}
`);

  write('build-profile.json5', `{
  "app": {
    "signingConfigs": [],
    "products": [
      {
        "name": "default",
        "signingConfig": "default",
        "compatibleSdkVersion": "5.0.0(12)",
        "runtimeOS": "HarmonyOS"
      }
    ],
    "buildModeSet": [
      { "name": "debug" },
      { "name": "release" }
    ]
  },
  "modules": [
    {
      "name": "entry",
      "srcPath": "./entry",
      "targets": [
        { "name": "default", "applyToProducts": ["default"] }
      ]
    }
  ]
}
`);

  write('hvigorfile.ts', `export { appTasks } from '@ohos/hvigor-ohos-plugin';
`);

  write('hvigor/hvigor-config.json5', `{
  "modelVersion": "5.0.0",
  "dependencies": {},
  "execution": {},
  "logging": {},
  "debugging": {},
  "nodeOptions": {}
}
`);

  write('.gitignore', `# HarmonyOS / DevEco build output
/build/
/entry/build/
/packages/*/build/
/oh_modules/
/entry/oh_modules/
/packages/*/oh_modules/
/.hvigor/
/.idea/
/.cxx/
local.properties
*.har
*.hap
*.hsp
*.app
`);

  write('AppScope/app.json5', `{
  "app": {
    "bundleName": "${bundleName}",
    "vendor": "SDKWork",
    "versionCode": 1000000,
    "versionName": "0.1.0",
    "label": "$string:app_name",
    "minAPIVersion": 12,
    "targetAPIVersion": 12,
    "apiReleaseType": "Release"
  }
}
`);

  writeJson('AppScope/resources/base/element/string.json', {
    string: [{ name: 'app_name', value: 'SDKWork Agents' }],
  });
}

// ---------------------------------------------------------------------------
// entry module (thin composition root)
// ---------------------------------------------------------------------------

function materializeEntry() {
  write('entry/oh-package.json5', `{
  "name": "entry",
  "version": "0.1.0",
  "description": "SDKWork Agents HarmonyOS entry and composition module",
  "main": "",
  "author": "SDKWork",
  "license": "Apache-2.0",
  "dependencies": {
    "@sdkwork/sdkwork-agents-harmony-mobile-core": "file:../packages/sdkwork-agents-harmony-mobile-core",
    "@sdkwork/sdkwork-agents-harmony-mobile-commons": "file:../packages/sdkwork-agents-harmony-mobile-commons",
    "@sdkwork/sdkwork-agents-harmony-mobile-shell": "file:../packages/sdkwork-agents-harmony-mobile-shell",
    "@sdkwork/sdkwork-agents-harmony-mobile-host": "file:../packages/sdkwork-agents-harmony-mobile-host",
    "@sdkwork/sdkwork-agents-harmony-mobile-agents": "file:../packages/sdkwork-agents-harmony-mobile-agents"
  }
}
`);

  write('entry/build-profile.json5', `{
  "apiType": "stageMode",
  "buildOption": {},
  "buildOptionSet": [
    {
      "name": "release",
      "arkOptions": {
        "obfuscation": {
          "ruleOptions": {
            "enable": false
          }
        }
      }
    }
  ],
  "targets": [
    { "name": "default" }
  ]
}
`);

  write('entry/src/main/module.json5', `{
  "module": {
    "name": "entry",
    "type": "entry",
    "description": "$string:module_desc",
    "mainElement": "EntryAbility",
    "deviceTypes": ["phone", "tablet"],
    "deliveryWithInstall": true,
    "installationFree": false,
    "pages": "$profile:main_pages",
    "abilities": [
      {
        "name": "EntryAbility",
        "srcEntry": "./ets/entryability/EntryAbility.ets",
        "description": "$string:EntryAbility_desc",
        "label": "$string:EntryAbility_label",
        "startWindowBackground": "$color:start_window_background",
        "exported": true,
        "skills": [
          {
            "entities": ["entity.system.home"],
            "actions": ["action.system.home"]
          }
        ]
      }
    ],
    "requestPermissions": [
      {
        "name": "ohos.permission.INTERNET"
      },
      {
        "name": "ohos.permission.GET_NETWORK_INFO"
      }
    ]
  }
}
`);

  write('entry/src/main/ets/entryability/EntryAbility.ets', `import { AbilityConstant, UIAbility, Want } from '@kit.AbilityKit';
import { window } from '@kit.ArkUI';
import { hilog } from '@kit.PerformanceAnalysisKit';

import { bootstrapAgentsHarmonyApp } from '../bootstrap/Runtime';

const DOMAIN: number = 0x0000;
const TAG: string = 'SdkworkAgentsEntry';

/**
 * HarmonyOS entry ability.
 *
 * This module stays thin by contract: it wires the bootstrap/composition
 * module and nothing else. Business pages live in capability packages.
 */
export default class EntryAbility extends UIAbility {
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(DOMAIN, TAG, 'onCreate');
    bootstrapAgentsHarmonyApp(this.context);
  }

  onDestroy(): void {
    hilog.info(DOMAIN, TAG, 'onDestroy');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(DOMAIN, TAG, 'loadContent failed: %{public}s', JSON.stringify(err));
        return;
      }
      hilog.info(DOMAIN, TAG, 'loadContent succeeded');
    });
  }

  onWindowStageDestroy(): void {
    hilog.info(DOMAIN, TAG, 'onWindowStageDestroy');
  }

  onForeground(): void {
    hilog.info(DOMAIN, TAG, 'onForeground');
  }

  onBackground(): void {
    hilog.info(DOMAIN, TAG, 'onBackground');
  }
}
`);

  write('entry/src/main/ets/bootstrap/Environment.ets', `/**
 * HarmonyOS runtime environment resolution.
 *
 * Authority: \`config/app/runtime-env.<deploymentProfile>.<environment>.json\`
 * (see \`CONFIG_SPEC.md\` and \`ENVIRONMENT_SPEC.md\`). The build projects exactly
 * one selected non-secret runtime JSON resource; this module only reads it.
 */

export type AgentsHarmonyRuntimeTarget = 'harmony-native';

export interface AgentsHarmonyEnvironment {
  readonly environment: string;
  readonly deploymentProfile: string;
  readonly profileId: string;
  readonly runtimeTarget: AgentsHarmonyRuntimeTarget;
  readonly agentsApiBaseUrl: string;
  readonly agentsBackendApiBaseUrl: string;
  readonly appbaseAppApiBaseUrl: string;
  readonly appbaseLoginUrl: string;
}

const APP_API_SUFFIX: string = '/app/v3/api';

function requireNonEmpty(value: string | undefined, key: string): string {
  const normalized: string = value === undefined ? '' : value.trim();
  if (normalized.length === 0) {
    throw new Error(\`\${key} is required by the HarmonyOS runtime config\`);
  }
  return normalized;
}

function stripTrailingSlashes(value: string): string {
  let normalized: string = value.trim();
  while (normalized.endsWith('/')) {
    normalized = normalized.slice(0, normalized.length - 1);
  }
  return normalized;
}

export function normalizeAgentsAppApiBaseUrl(value: string): string {
  const normalized: string = stripTrailingSlashes(requireNonEmpty(value, 'agents.apiBaseUrl'));
  if (!normalized.endsWith(APP_API_SUFFIX)) {
    throw new Error(\`agents.apiBaseUrl must end with \${APP_API_SUFFIX}\`);
  }
  const prefix: string = normalized.slice(0, normalized.length - APP_API_SUFFIX.length);
  if (prefix.endsWith(APP_API_SUFFIX)) {
    throw new Error(\`agents.apiBaseUrl must contain \${APP_API_SUFFIX} exactly once\`);
  }
  return normalized;
}

export function createAgentsHarmonyEnvironment(
  runtimeConfig: Record<string, Object>,
): AgentsHarmonyEnvironment {
  const agents: Record<string, Object> = runtimeConfig.agents as Record<string, Object>;
  const appbase: Record<string, Object> = runtimeConfig.appbase as Record<string, Object>;
  const target: string = requireNonEmpty(runtimeConfig.runtimeTarget as string, 'runtimeTarget');
  if (target !== 'harmony-native') {
    throw new Error('runtimeTarget must be harmony-native for this application root');
  }
  return {
    environment: requireNonEmpty(runtimeConfig.environment as string, 'environment'),
    deploymentProfile: requireNonEmpty(runtimeConfig.deploymentProfile as string, 'deploymentProfile'),
    profileId: requireNonEmpty(runtimeConfig.profileId as string, 'profileId'),
    runtimeTarget: 'harmony-native',
    agentsApiBaseUrl: normalizeAgentsAppApiBaseUrl(agents.apiBaseUrl as string),
    agentsBackendApiBaseUrl: requireNonEmpty(agents.backendApiBaseUrl as string, 'agents.backendApiBaseUrl'),
    appbaseAppApiBaseUrl: requireNonEmpty(appbase.appApiBaseUrl as string, 'appbase.appApiBaseUrl'),
    appbaseLoginUrl: requireNonEmpty(appbase.loginUrl as string, 'appbase.loginUrl'),
  };
}
`);

  write('entry/src/main/ets/bootstrap/SdkClients.ets', `import {
  configureAgentsAppSdkBaseUrl,
  getAgentsAppSdkClient,
  resetAgentsAppSdkClient,
  type AgentsAppSdkClient,
  type AgentsAppSdkClientConfig,
} from '@sdkwork/sdkwork-agents-harmony-mobile-core';

/**
 * SDK client construction for the HarmonyOS root.
 *
 * Authority: \`APP_SDK_INTEGRATION_SPEC.md\` and
 * \`HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 6. Concrete clients are
 * constructed here (bootstrap only) and injected into feature packages.
 */
export class AgentsHarmonySdkClients {
  private agentsClient: AgentsAppSdkClient | null = null;

  initialize(appApiBaseUrl: string, accessToken?: string): void {
    configureAgentsAppSdkBaseUrl(appApiBaseUrl);
    const config: AgentsAppSdkClientConfig = {
      baseUrl: appApiBaseUrl,
      accessToken: accessToken,
    };
    this.agentsClient = getAgentsAppSdkClient(config);
  }

  get agents(): AgentsAppSdkClient {
    if (this.agentsClient === null) {
      throw new Error('Agents app SDK client must be initialized by bootstrap before use');
    }
    return this.agentsClient;
  }

  reset(): void {
    resetAgentsAppSdkClient();
    this.agentsClient = null;
  }
}

let sdkClients: AgentsHarmonySdkClients | null = null;

export function createAgentsHarmonySdkClients(): AgentsHarmonySdkClients {
  sdkClients = new AgentsHarmonySdkClients();
  return sdkClients;
}

export function getAgentsHarmonySdkClients(): AgentsHarmonySdkClients {
  if (sdkClients === null) {
    return createAgentsHarmonySdkClients();
  }
  return sdkClients;
}
`);

  write('entry/src/main/ets/bootstrap/IamRuntime.ets', `/**
 * Appbase IAM runtime wiring for the HarmonyOS root.
 *
 * Authority: \`APP_SDK_INTEGRATION_SPEC.md\` and \`IAM_LOGIN_INTEGRATION_SPEC.md\`.
 *
 * The global token-manager equivalent is shared by the appbase app SDK, every
 * authenticated app-api SDK client, and explicit backend-admin clients.
 * Logout and refresh failure must clear the token manager, session/context
 * stores, secure platform storage, and realtime bridges.
 */

export interface AgentsHarmonyIamRuntime {
  isAuthenticated(): boolean;
  clearOnLogout(): void;
}

export class AgentsHarmonyIamRuntimeImpl implements AgentsHarmonyIamRuntime {
  private authenticated: boolean = false;

  isAuthenticated(): boolean {
    return this.authenticated;
  }

  clearOnLogout(): void {
    this.authenticated = false;
  }
}

let iamRuntime: AgentsHarmonyIamRuntime | null = null;

export function createAgentsHarmonyIamRuntime(): AgentsHarmonyIamRuntime {
  iamRuntime = new AgentsHarmonyIamRuntimeImpl();
  return iamRuntime;
}

export function getAgentsHarmonyIamRuntime(): AgentsHarmonyIamRuntime {
  if (iamRuntime === null) {
    return createAgentsHarmonyIamRuntime();
  }
  return iamRuntime;
}
`);

  write('entry/src/main/ets/bootstrap/HostAdapters.ets', `import { AgentsHarmonyHostAdapters, createHostAdapters } from '@sdkwork/sdkwork-agents-harmony-mobile-host';

/**
 * Host adapter registration.
 *
 * Authority: \`HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 7. Feature
 * packages depend on these typed interfaces and never call HarmonyOS system
 * APIs directly.
 */

let hostAdapters: AgentsHarmonyHostAdapters | null = null;

export function registerAgentsHarmonyHostAdapters(): AgentsHarmonyHostAdapters {
  hostAdapters = createHostAdapters();
  return hostAdapters;
}

export function getAgentsHarmonyHostAdapters(): AgentsHarmonyHostAdapters {
  if (hostAdapters === null) {
    return registerAgentsHarmonyHostAdapters();
  }
  return hostAdapters;
}
`);

  write('entry/src/main/ets/bootstrap/Routes.ets', `import { agentsRouteContributions, type AgentsRouteContribution } from '@sdkwork/sdkwork-agents-harmony-mobile-agents';

/**
 * Route/page registry assembly.
 *
 * Route ids follow \`<surface>.<domain>.<capability>.<screen>\` and stay aligned
 * with the PC and H5 roots (\`APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md\`
 * section 7). Physical Harmony page paths may differ from other platforms.
 */

export function createAgentsHarmonyRoutes(): AgentsRouteContribution[] {
  return agentsRouteContributions;
}
`);

  write('entry/src/main/ets/bootstrap/Runtime.ets', `import { AbilityContext } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';

import { createAgentsHarmonyEnvironment, type AgentsHarmonyEnvironment } from './Environment';
import { registerAgentsHarmonyHostAdapters } from './HostAdapters';
import { createAgentsHarmonyIamRuntime } from './IamRuntime';
import { createAgentsHarmonySdkClients } from './SdkClients';
import { createAgentsHarmonyRoutes } from './Routes';

const DOMAIN: number = 0x0000;
const TAG: string = 'SdkworkAgentsBootstrap';

/**
 * Root composition entry.
 *
 * This module assembles the environment, SDK clients, IAM runtime, host
 * adapters, and route registry only. It must not own product business logic.
 */
export interface AgentsHarmonyRuntime {
  readonly environment: AgentsHarmonyEnvironment;
  readonly routeCount: number;
}

export function bootstrapAgentsHarmonyApp(context: AbilityContext): AgentsHarmonyRuntime | null {
  try {
    const environment: AgentsHarmonyEnvironment = createAgentsHarmonyEnvironment(
      createAgentsHarmonyRuntimeConfig(),
    );
    createAgentsHarmonySdkClients().initialize(environment.agentsApiBaseUrl);
    createAgentsHarmonyIamRuntime();
    registerAgentsHarmonyHostAdapters();
    const routes = createAgentsHarmonyRoutes();
    hilog.info(DOMAIN, TAG, 'bootstrap completed for profile %{public}s', environment.profileId);
    return { environment, routeCount: routes.length };
  } catch (error) {
    hilog.error(DOMAIN, TAG, 'bootstrap failed: %{public}s', JSON.stringify(error));
    return null;
  }
}

/**
 * Runtime config projection.
 *
 * The build projects exactly one non-secret
 * \`config/app/runtime-env.<deploymentProfile>.<environment>.json\` resource
 * into the HAP. Until the resource projection task is wired into the hvigor
 * pipeline, bootstrap fails fast instead of falling back to a hand-written
 * environment.
 */
function createAgentsHarmonyRuntimeConfig(): Record<string, Object> {
  throw new Error(
    'Harmony runtime config projection is not wired yet: provide config/app/runtime-env.<profile-id>.json as an ArkTS resource module',
  );
}
`);

  write('entry/src/main/ets/pages/Index.ets', `import { getAgentsHarmonySdkClients } from '../bootstrap/SdkClients';

/**
 * Root page.
 *
 * The root page stays a thin shell mount point. Real screens are contributed by
 * capability packages (\`packages/sdkwork-agents-harmony-mobile-*\`).
 */
@Entry
@Component
struct Index {
  @State message: string = 'SDKWork Agents';

  aboutToAppear(): void {
    getAgentsHarmonySdkClients();
  }

  build() {
    Column() {
      Text(this.message)
        .fontSize(24)
        .fontWeight(FontWeight.Bold)
      Text('Harmony shell mount point')
        .fontSize(14)
        .fontColor('#666666')
        .margin({ top: 8 })
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
  }
}
`);

  writeJson('entry/src/main/resources/base/element/string.json', {
    string: [
      { name: 'module_desc', value: 'SDKWork Agents HarmonyOS entry module' },
      { name: 'EntryAbility_desc', value: 'SDKWork Agents' },
      { name: 'EntryAbility_label', value: 'SDKWork Agents' },
    ],
  });

  writeJson('entry/src/main/resources/base/element/color.json', {
    color: [{ name: 'start_window_background', value: '#FFFFFF' }],
  });

  writeJson('entry/src/main/resources/base/profile/main_pages.json', {
    src: ['pages/Index'],
  });

  write('entry/src/ohosTest/ets/test/Ability.test.ets', `import { describe, expect, it } from '@ohos/hypium';

/**
 * HarmonyOS module smoke test.
 *
 * Runs under \`hvigor test\` with the HarmonyOS SDK available.
 */
export default function abilityTest() {
  describe('EntryAbilityTest', () => {
    it('module scaffold is loadable', 0, () => {
      expect(true).assertTrue();
    });
  });
}
`);
}

// ---------------------------------------------------------------------------
// Package family
// ---------------------------------------------------------------------------

function sdkworkSpecsForPackage(archSpecFile, extra) {
  const base = [
    {
      file: archSpecFile,
      path: '../../../../../sdkwork-specs/' + archSpecFile,
      purpose: 'HarmonyOS mobile application root architecture.',
    },
    {
      file: 'APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md',
      path: '../../../../../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md',
      purpose: 'Cross-client package taxonomy and dependency direction.',
    },
  ];
  return base.concat(extra);
}

function packageComponentSpec({ packageName, displayName, capability, layerRole, extraContracts, extraSpecs }) {
  return {
    schemaVersion: 1,
    kind: 'sdkwork.component.spec',
    component: {
      name: packageName,
      displayName,
      version: '0.1.0',
      type: 'arkts-package',
      root: `apps/${appRootName}/packages/${packageName}`,
      domain: applicationCode,
      capability,
      surface: 'app',
      languages: ['arkts'],
      generated: false,
      manifests: ['oh-package.json5', 'specs/component.spec.json'],
    },
    canonicalSpecs: sdkworkSpecsForPackage('HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md', extraSpecs ?? []),
    contracts: {
      layerRole,
      publicExports: ['Index.ets'],
      sdkClients: [],
      sdkDependencies: [],
      dependencyApiExports: [],
      dependencyApiSurfaces: [],
      ...(extraContracts ?? {}),
    },
    verification: {
      commands: ['node ../sdkwork-specs/tools/check-frontend-composition.mjs --root .'],
    },
  };
}

function packageOhPackageJson(packageName, description) {
  return `{
  "name": "@sdkwork/${packageName}",
  "version": "0.1.0",
  "description": "${description}",
  "main": "src/main/ets/Index.ets",
  "author": "SDKWork",
  "license": "Apache-2.0",
  "dependencies": {}
}
`;
}

function packageBuildProfile() {
  return `{
  "apiType": "stageMode",
  "buildOption": {},
  "buildOptionSet": [
    {
      "name": "release",
      "arkOptions": {
        "obfuscation": {
          "ruleOptions": {
            "enable": false
          }
        }
      }
    }
  ],
  "targets": [
    { "name": "default" }
  ]
}
`;
}

function packageModuleJson5(moduleName) {
  return `{
  "module": {
    "name": "${moduleName}",
    "type": "har",
    "deviceTypes": ["phone", "tablet"]
  }
}
`;
}

function materializePackage(packageName, { description, moduleName, indexPath, indexContent, componentSpec, compositionContractJson }) {
  const base = `packages/${packageName}`;
  write(`${base}/oh-package.json5`, packageOhPackageJson(packageName, description));
  write(`${base}/build-profile.json5`, packageBuildProfile());
  write(`${base}/src/main/module.json5`, packageModuleJson5(moduleName));
  write(`${base}/README.md`, `# ${packageName}

${description}

Authority: \`../../../../sdkwork-specs/HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\`.
`);
  writeJson(`${base}/specs/component.spec.json`, componentSpec);
  if (compositionContractJson !== undefined) {
    writeJson(`${base}/package.json`, compositionContractJson);
  }
  for (const entry of indexPath) {
    write(`${base}/${entry.path}`, entry.content);
  }
  if (indexContent !== undefined) {
    write(`${base}/src/main/ets/Index.ets`, indexContent);
  }
}

/**
 * Cross-architecture composition contract entry for a HarmonyOS core package.
 *
 * `check-frontend-composition` recognizes `src/composition/` plus
 * `package.json`/`pubspec.yaml` as the core composition contract. ArkTS
 * packages ship `oh-package.json5` for the Harmony toolchain, which that check
 * does not know about yet. This manifest therefore declares the composition
 * subpath contract over the real ArkTS sources and does not stand in for a
 * Node/pnpm package: the Harmony root is an ohpm workspace and is deliberately
 * absent from the repository `pnpm-workspace.yaml`.
 */
function harmonyCoreCompositionContract(packageName, subpaths) {
  return {
    name: `@sdkwork/${packageName}`,
    private: true,
    version: '0.1.0',
    type: 'module',
    sdkwork: {
      role: 'harmony-arkts-composition-contract',
      toolchain: 'ohpm',
      note: 'Composition contract entry for check-frontend-composition. Not a pnpm workspace member.',
    },
    exports: subpaths,
  };
}

function materializePackages() {
  // ---- core -------------------------------------------------------------
  materializePackage('sdkwork-agents-harmony-mobile-core', {
    description: 'SDKWork Agents HarmonyOS mobile core: runtime config, SDK factories, token manager, session stores, route registry, host adapter contracts.',
    moduleName: 'sdkwork_agents_harmony_mobile_core',
    componentSpec: packageComponentSpec({
      packageName: 'sdkwork-agents-harmony-mobile-core',
      displayName: 'SDKWork Agents HarmonyOS Mobile Core',
      capability: 'core',
      layerRole: 'frontend-core',
      extraSpecs: [
        {
          file: 'APP_COMPOSITION_SPEC.md',
          path: '../../../../../sdkwork-specs/APP_COMPOSITION_SPEC.md',
          purpose: 'Native-authority application composition.',
        },
        {
          file: 'APP_SDK_INTEGRATION_SPEC.md',
          path: '../../../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md',
          purpose: 'Generated ArkTS/TypeScript app SDK integration.',
        },
      ],
      extraContracts: {
        layerRole: 'frontend-core',
        sdkClients: ['SdkworkAppClient'],
        sdkDependencies: [
          {
            workspace: 'sdkwork-agents-app-sdk',
            surface: 'app-api',
            credentialMode: 'authenticated-app-api',
            status: 'active',
            runtimeAdaptation: 'arkts-pending',
          },
        ],
      },
    }),
    indexPath: [
      {
        path: 'src/main/ets/sdk/AgentsAppSdkClient.ets',
        content: `/**
 * Agents app-api SDK port and factory contract.
 *
 * Authority: \`APP_SDK_INTEGRATION_SPEC.md\` and
 * \`HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 6.
 *
 * The Harmony root consumes \`/app/v3/api\` through a generated
 * ArkTS/TypeScript app SDK client adapted for the Harmony runtime. This file
 * owns the injected port contract, the base-URL normalization, and the
 * credential resolution boundary. Concrete transport construction stays in
 * the root bootstrap.
 *
 * PREREQUISITE: the SDK generation chain currently emits TypeScript and
 * Flutter targets of \`sdkwork-agents-app-sdk\` only. No ArkTS target exists
 * yet, so this module deliberately declares the port and the adapter seam
 * instead of vendoring a transport copy. Feature packages must never fill
 * this gap with raw request APIs or manual auth headers.
 */

export interface AgentsAppSdkClientConfig {
  readonly baseUrl: string;
  readonly accessToken?: string;
  readonly authToken?: string;
  readonly platform?: string;
}

export interface AgentsAppSdkClient {
  readonly baseUrl: string;
  readonly platform: string;
}

const APP_API_SUFFIX: string = '/app/v3/api';

let configuredBaseUrl: string | null = null;

export function configureAgentsAppSdkBaseUrl(baseUrl: string): void {
  const normalized: string = baseUrl.trim().replace(/\\/+$/u, '');
  if (normalized.length === 0) {
    throw new Error('agents app API base URL is required before SDK bootstrap');
  }
  if (!normalized.endsWith(APP_API_SUFFIX)) {
    throw new Error(\`agents app API base URL must end with \${APP_API_SUFFIX}\`);
  }
  configuredBaseUrl = normalized;
}

export function resolveAgentsAppSdkBaseUrl(): string {
  if (configuredBaseUrl === null) {
    throw new Error('agents app API base URL must be configured before SDK bootstrap');
  }
  return configuredBaseUrl;
}

export function createAgentsAppSdkClientConfig(
  baseUrl: string,
  accessToken?: string,
  authToken?: string,
): AgentsAppSdkClientConfig {
  configureAgentsAppSdkBaseUrl(baseUrl);
  return {
    baseUrl: resolveAgentsAppSdkBaseUrl(),
    accessToken,
    authToken,
    platform: 'harmony-native',
  };
}

const agentsAppSdkClient: AgentsAppSdkClient | null = null;

export function getAgentsAppSdkClient(config?: AgentsAppSdkClientConfig): AgentsAppSdkClient {
  const resolved: AgentsAppSdkClientConfig = config ?? createAgentsAppSdkClientConfig(resolveAgentsAppSdkBaseUrl());
  return agentsAppSdkClient ?? {
    baseUrl: resolved.baseUrl,
    platform: resolved.platform ?? 'harmony-native',
  };
}

export function resetAgentsAppSdkClient(): void {
  configuredBaseUrl = null;
}
`,
      },
      {
        path: 'src/main/ets/sdk/SdkInventory.ets',
        content: `/**
 * SDK client packages consumed by Harmony core.
 *
 * Authority: \`specs/component.spec.json\` \`contracts.sdkDependencies\`.
 */
export const agentsHarmonyCoreSdkInventory: string[] = [
  'sdkwork-agents-app-sdk',
];

export function listSdkworkCoreSdkInventory(): string[] {
  return agentsHarmonyCoreSdkInventory;
}
`,
      },
      {
        path: 'src/main/ets/session/SessionStore.ets',
        content: `/**
 * Session and context store.
 *
 * Authority: \`APP_SDK_INTEGRATION_SPEC.md\`. Logout, refresh failure, tenant
 * switch, and account switch must clear this store together with the token
 * manager, secure platform storage, and realtime/session bridges.
 */

export interface AgentsHarmonySession {
  readonly accessToken?: string;
  readonly authToken?: string;
  readonly tenantId?: string;
}

let currentSession: AgentsHarmonySession | null = null;

export function readAgentsHarmonySession(): AgentsHarmonySession | null {
  return currentSession;
}

export function writeAgentsHarmonySession(session: AgentsHarmonySession | null): void {
  currentSession = session;
}

export function clearAgentsHarmonySession(): void {
  currentSession = null;
}
`,
      },
      {
        path: 'src/main/ets/host/HostAdapterContracts.ets',
        content: `/**
 * Host adapter contracts owned by core and implemented by the host package.
 *
 * Authority: \`HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 7. Host
 * adapters expose stable user-safe errors only.
 */

export type HostAdapterError = 'unsupported' | 'permission-denied' | 'unavailable' | 'cancelled' | 'invalid-state';

export interface HostAdapterResult<T> {
  readonly ok: boolean;
  readonly value?: T;
  readonly error?: HostAdapterError;
}

export interface SecureStorageAdapter {
  get(key: string): Promise<HostAdapterResult<string>>;
  set(key: string, value: string): Promise<HostAdapterResult<void>>;
  remove(key: string): Promise<HostAdapterResult<void>>;
  clear(): Promise<HostAdapterResult<void>>;
}

export interface NetworkStatusAdapter {
  isOnline(): Promise<HostAdapterResult<boolean>>;
}

export interface AppLifecycleAdapter {
  currentState(): Promise<HostAdapterResult<string>>;
}
`,
      },
      {
        path: 'src/main/ets/composition/DependencyManifest.ets',
        content: `export const sdkworkComponentSpecPath: string = '../../../specs/component.spec.json';
`,
      },
      {
        path: 'src/composition/index.ets',
        content: `/**
 * Cross-architecture composition entry.
 *
 * Authority: \`APP_COMPOSITION_SPEC.md\`. Feature packages resolve runtime
 * composition metadata through this core package's public exports only.
 */
export * from '../main/ets/composition/DependencyManifest';
`,
      },
      {
        path: 'src/main/ets/composition/SdkInventory.ets',
        content: `export { agentsHarmonyCoreSdkInventory, listSdkworkCoreSdkInventory } from '../sdk/SdkInventory';
`,
      },
      {
        path: 'src/main/ets/composition/ModuleRegistry.ets',
        content: `import { agentsRouteContributions } from '@sdkwork/sdkwork-agents-harmony-mobile-agents';

/**
 * Module registry.
 *
 * The root bootstrap reads this registry instead of importing capability
 * packages directly.
 */
export interface AgentsHarmonyModuleRegistration {
  readonly id: string;
  readonly routeIdPrefix: string;
}

export function listAgentsHarmonyModules(): AgentsHarmonyModuleRegistration[] {
  return [
    { id: 'agents', routeIdPrefix: 'app.agents.' },
  ];
}

export function listAgentsHarmonyModuleRouteCount(): number {
  return agentsRouteContributions.length;
}
`,
      },
      {
        path: 'src/main/ets/composition/HostRegistry.ets',
        content: `/**
 * Host adapter capability registry.
 *
 * Renderer/feature code routes through the declared capability set; direct
 * platform branches on HarmonyOS globals are forbidden.
 */
export const agentsHarmonyHostCapabilities: string[] = [
  'secureStorage',
  'networkStatus',
  'appLifecycle',
  'deepLinks',
  'filePicker',
  'camera',
  'qrScanner',
  'pushNotifications',
  'clipboard',
  'deviceInfo',
];

export function hasAgentsHarmonyHostCapability(capability: string): boolean {
  return agentsHarmonyHostCapabilities.includes(capability);
}
`,
      },
    ],
    indexContent: `/**
 * SDKWork Agents HarmonyOS mobile core public integration boundary.
 */
export * from './sdk/AgentsAppSdkClient';
export * from './sdk/SdkInventory';
export * from './session/SessionStore';
export * from './host/HostAdapterContracts';
export * from './composition/DependencyManifest';
export * from './composition/HostRegistry';
export * from './composition/ModuleRegistry';
`,
    compositionContractJson: harmonyCoreCompositionContract('sdkwork-agents-harmony-mobile-core', {
      '.': './src/main/ets/Index.ets',
      './sdk': './src/main/ets/sdk/SdkInventory.ets',
      './modules': './src/main/ets/composition/ModuleRegistry.ets',
      './host': './src/main/ets/host/HostAdapterContracts.ets',
      './session': './src/main/ets/session/SessionStore.ets',
      './composition': './src/composition/index.ets',
    }),
  });

  // ---- commons ----------------------------------------------------------
  materializePackage('sdkwork-agents-harmony-mobile-commons', {
    description: 'SDKWork Agents HarmonyOS mobile commons: domain-neutral ArkUI primitives, theme adapters, and i18n helpers.',
    moduleName: 'sdkwork_agents_harmony_mobile_commons',
    componentSpec: packageComponentSpec({
      packageName: 'sdkwork-agents-harmony-mobile-commons',
      displayName: 'SDKWork Agents HarmonyOS Mobile Commons',
      capability: 'commons',
      layerRole: 'frontend-commons',
      extraSpecs: [
        {
          file: 'FRONTEND_SPEC.md',
          path: '../../../../../sdkwork-specs/FRONTEND_SPEC.md',
          purpose: 'Frontend package layering.',
        },
        {
          file: 'APP_HARMONY_NATIVE_UI_SPEC.md',
          path: '../../../../../sdkwork-specs/APP_HARMONY_NATIVE_UI_SPEC.md',
          purpose: 'Harmony native ArkUI package rules.',
        },
      ],
    }),
    indexPath: [
      {
        path: 'src/main/ets/theme/DesignTokens.ets',
        content: `/**
 * Domain-neutral design tokens.
 *
 * Authority: \`APP_HARMONY_NATIVE_UI_SPEC.md\`. Shared packages must not import
 * application app shells.
 */
export class SdkworkAgentsHarmonyTokens {
  static readonly colorPrimary: string = '#0F766E';
  static readonly colorBackground: string = '#F8FAFC';
  static readonly colorText: string = '#0F172A';
  static readonly colorTextMuted: string = '#64748B';
  static readonly spacingSm: number = 8;
  static readonly spacingMd: number = 16;
  static readonly spacingLg: number = 24;
}
`,
      },
      {
        path: 'src/main/ets/components/EmptyState.ets',
        content: `/**
 * Domain-neutral empty/error/retry primitive.
 */
@Component
export struct EmptyState {
  @Prop message: string = '';

  build() {
    Column() {
      Text(this.message)
        .fontSize(14)
        .fontColor('#64748B')
    }
    .width('100%')
    .padding(24)
    .justifyContent(FlexAlign.Center)
  }
}
`,
      },
      {
        path: 'src/main/ets/i18n/I18nHelpers.ets',
        content: `/**
 * Package-local i18n resource helpers.
 *
 * Authority: \`I18N_SPEC.md\`. Platform monolithic resources must be assembled
 * from fragments and must not be hand-authored whole-root catalogs.
 */
export type AgentsHarmonyLocale = 'en-US' | 'zh-CN';

export function normalizeAgentsHarmonyLocale(value: string): AgentsHarmonyLocale {
  return value.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}
`,
      },
    ],
    indexContent: `/**
 * SDKWork Agents HarmonyOS mobile commons public integration boundary.
 */
export * from './theme/DesignTokens';
export * from './components/EmptyState';
export * from './i18n/I18nHelpers';
`,
  });

  // ---- shell ------------------------------------------------------------
  materializePackage('sdkwork-agents-harmony-mobile-shell', {
    description: 'SDKWork Agents HarmonyOS mobile shell: app surface shell, navigation container, and AuthGate integration.',
    moduleName: 'sdkwork_agents_harmony_mobile_shell',
    componentSpec: packageComponentSpec({
      packageName: 'sdkwork-agents-harmony-mobile-shell',
      displayName: 'SDKWork Agents HarmonyOS Mobile Shell',
      capability: 'shell',
      layerRole: 'frontend-shell',
      extraSpecs: [
        {
          file: 'FRONTEND_SPEC.md',
          path: '../../../../../sdkwork-specs/FRONTEND_SPEC.md',
          purpose: 'Frontend package layering and SDK injection boundaries.',
        },
      ],
    }),
    indexPath: [
      {
        path: 'src/main/ets/navigation/RouteStack.ets',
        content: `/**
 * Page/route stack assembly for the HarmonyOS shell.
 *
 * Route ids follow \`<surface>.<domain>.<capability>.<screen>\`. Physical Harmony
 * page paths may differ from other platforms while route ids stay aligned.
 */
export interface AgentsHarmonyRouteRegistration {
  readonly id: string;
  readonly pagePath: string;
  readonly titleKey: string;
  readonly auth: 'public' | 'required';
}

export function createAgentsHarmonyRouteStack(routes: AgentsHarmonyRouteRegistration[]): AgentsHarmonyRouteRegistration[] {
  return routes.filter((route) => route.pagePath.trim().length > 0);
}
`,
      },
      {
        path: 'src/main/ets/auth/AuthGate.ets',
        content: `/**
 * AuthGate integration.
 *
 * Route guards are shell/runtime responsibilities. Capability packages declare
 * auth mode and permission hints only.
 */
export interface AgentsHarmonyAuthGateDecision {
  readonly allowed: boolean;
  readonly reason?: string;
}

export function evaluateAgentsHarmonyAuthGate(
  auth: 'public' | 'required',
  isAuthenticated: boolean,
): AgentsHarmonyAuthGateDecision {
  if (auth === 'public' || isAuthenticated) {
    return { allowed: true };
  }
  return { allowed: false, reason: 'authentication-required' };
}
`,
      },
    ],
    indexContent: `/**
 * SDKWork Agents HarmonyOS mobile shell public integration boundary.
 */
export * from './navigation/RouteStack';
export * from './auth/AuthGate';
`,
  });

  // ---- host -------------------------------------------------------------
  materializePackage('sdkwork-agents-harmony-mobile-host', {
    description: 'SDKWork Agents HarmonyOS mobile host: typed HarmonyOS platform adapters behind core-owned contracts.',
    moduleName: 'sdkwork_agents_harmony_mobile_host',
    componentSpec: packageComponentSpec({
      packageName: 'sdkwork-agents-harmony-mobile-host',
      displayName: 'SDKWork Agents HarmonyOS Mobile Host',
      capability: 'host',
      layerRole: 'frontend-host',
      extraSpecs: [
        {
          file: 'FRONTEND_SPEC.md',
          path: '../../../../../sdkwork-specs/FRONTEND_SPEC.md',
          purpose: 'Frontend package layering and host boundaries.',
        },
      ],
    }),
    indexPath: [
      {
        path: 'src/main/ets/HostAdapters.ets',
        content: `import {
  type AppLifecycleAdapter,
  type HostAdapterResult,
  type NetworkStatusAdapter,
  type SecureStorageAdapter,
} from '@sdkwork/sdkwork-agents-harmony-mobile-core';

/**
 * HarmonyOS host adapter implementations.
 *
 * Authority: \`HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 7. Adapters
 * expose typed methods and stable user-safe errors. They must not own login,
 * token refresh, permission evaluation, business authorization, or raw
 * business API transport.
 *
 * The platform-backed implementations require the HarmonyOS SDK toolchain and
 * concrete ability context injection; the capability set and method signatures
 * are fixed here so feature code compiles unchanged against any host.
 */

export class HarmonySecureStorageAdapter implements SecureStorageAdapter {
  get(key: string): Promise<HostAdapterResult<string>> {
    return Promise.resolve({ ok: false, error: 'unsupported' });
  }

  set(key: string, value: string): Promise<HostAdapterResult<void>> {
    return Promise.resolve({ ok: false, error: 'unsupported' });
  }

  remove(key: string): Promise<HostAdapterResult<void>> {
    return Promise.resolve({ ok: false, error: 'unsupported' });
  }

  clear(): Promise<HostAdapterResult<void>> {
    return Promise.resolve({ ok: false, error: 'unsupported' });
  }
}

export class HarmonyNetworkStatusAdapter implements NetworkStatusAdapter {
  isOnline(): Promise<HostAdapterResult<boolean>> {
    return Promise.resolve({ ok: true, value: true });
  }
}

export class HarmonyAppLifecycleAdapter implements AppLifecycleAdapter {
  currentState(): Promise<HostAdapterResult<string>> {
    return Promise.resolve({ ok: true, value: 'foreground' });
  }
}

export interface AgentsHarmonyHostAdapters {
  readonly capabilities: string[];
  readonly secureStorage: SecureStorageAdapter;
  readonly networkStatus: NetworkStatusAdapter;
  readonly appLifecycle: AppLifecycleAdapter;
}

export function createHostAdapters(): AgentsHarmonyHostAdapters {
  return {
    capabilities: ['networkStatus', 'appLifecycle'],
    secureStorage: new HarmonySecureStorageAdapter(),
    networkStatus: new HarmonyNetworkStatusAdapter(),
    appLifecycle: new HarmonyAppLifecycleAdapter(),
  };
}
`,
      },
    ],
    indexContent: `/**
 * SDKWork Agents HarmonyOS mobile host public integration boundary.
 */
export * from './HostAdapters';
`,
  });

  // ---- agents capability ------------------------------------------------
  materializePackage('sdkwork-agents-harmony-mobile-agents', {
    description: 'SDKWork Agents HarmonyOS mobile agents capability: agent catalog, creation, and conversation screens.',
    moduleName: 'sdkwork_agents_harmony_mobile_agents',
    componentSpec: packageComponentSpec({
      packageName: 'sdkwork-agents-harmony-mobile-agents',
      displayName: 'SDKWork Agents HarmonyOS Mobile Agents',
      capability: 'agents',
      layerRole: 'frontend-feature',
      extraSpecs: [
        {
          file: 'APP_HARMONY_NATIVE_UI_SPEC.md',
          path: '../../../../../sdkwork-specs/APP_HARMONY_NATIVE_UI_SPEC.md',
          purpose: 'Harmony native ArkUI capability package rules.',
        },
        {
          file: 'PAGINATION_SPEC.md',
          path: '../../../../../sdkwork-specs/PAGINATION_SPEC.md',
          purpose: 'Interactive list pagination.',
        },
        {
          file: 'I18N_SPEC.md',
          path: '../../../../../sdkwork-specs/I18N_SPEC.md',
          purpose: 'Runtime locale and message catalog rules.',
        },
      ],
      extraContracts: {
        layerRole: 'frontend-feature',
        providedPorts: [
          { name: 'agentsHarmonyCatalogViews', export: 'Index.ets' },
          { name: 'agentsHarmonyRoutes', export: 'Index.ets' },
        ],
        requiredPorts: [
          { name: 'agentsAppSdkClient', package: '@sdkwork/sdkwork-agents-harmony-mobile-core', export: 'src/main/ets/sdk/AgentsAppSdkClient.ets' },
          { name: 'agentsHarmonyCommons', package: '@sdkwork/sdkwork-agents-harmony-mobile-commons', export: 'Index.ets' },
        ],
        sdkClients: [],
        sdkDependencies: ['sdkwork-agents-app-sdk'],
      },
    }),
    indexPath: [
      {
        path: 'src/main/ets/routes/RouteContributions.ets',
        content: `/**
 * Route contributions for the agents capability.
 *
 * Route ids follow \`<surface>.<domain>.<capability>.<screen>\` and are aligned
 * with the PC and H5 roots. Route metadata must not declare HTTP API paths,
 * SDK methods, raw URL constants, or transport details.
 */
export interface AgentsRouteContribution {
  readonly id: string;
  readonly surface: 'app';
  readonly domain: string;
  readonly capability: string;
  readonly screen: string;
  readonly pagePath: string;
  readonly titleKey: string;
  readonly auth: 'public' | 'required';
  readonly permissionHint?: string;
}

export const agentsRouteContributions: AgentsRouteContribution[] = [
  {
    id: 'app.agents.catalog.list',
    surface: 'app',
    domain: 'agents',
    capability: 'catalog',
    screen: 'list',
    pagePath: 'pages/agents/AgentsList',
    titleKey: 'agents.catalog.title',
    auth: 'required',
    permissionHint: 'agents.agents.read',
  },
  {
    id: 'app.agents.catalog.detail',
    surface: 'app',
    domain: 'agents',
    capability: 'catalog',
    screen: 'detail',
    pagePath: 'pages/agents/AgentDetail',
    titleKey: 'agents.detail.title',
    auth: 'required',
    permissionHint: 'agents.agents.read',
  },
  {
    id: 'app.agents.catalog.create',
    surface: 'app',
    domain: 'agents',
    capability: 'catalog',
    screen: 'create',
    pagePath: 'pages/agents/CreateAgent',
    titleKey: 'agents.create.title',
    auth: 'required',
    permissionHint: 'agents.agents.write',
  },
  {
    id: 'app.agents.conversation.chat',
    surface: 'app',
    domain: 'agents',
    capability: 'conversation',
    screen: 'chat',
    pagePath: 'pages/agents/AgentConversation',
    titleKey: 'agents.conversation.title',
    auth: 'required',
    permissionHint: 'agents.agents.read',
  },
];
`,
      },
      {
        path: 'src/main/ets/models/AgentModels.ets',
        content: `/**
 * View/screen models for the agents capability.
 *
 * API DTOs come from generated ArkTS/TypeScript SDKs; this file owns view
 * models, screen models, and route params only.
 */
export interface AgentsCatalogListItem {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  readonly avatarUrl?: string;
}

export interface AgentsConversationRouteParams {
  readonly agentId: string;
  readonly sessionId?: string;
}

export interface AgentsCatalogScreenModel {
  readonly items: AgentsCatalogListItem[];
  readonly loading: boolean;
  readonly errorMessage?: string;
  readonly hasMore: boolean;
}
`,
      },
      {
        path: 'src/main/ets/services/AgentCatalogService.ets',
        content: `import { type AgentsAppSdkClient } from '@sdkwork/sdkwork-agents-harmony-mobile-core';

import { type AgentsCatalogListItem } from '../models/AgentModels';

/**
 * Use-case orchestration for the agents catalog.
 *
 * Services receive explicit runtime inputs (injected SDK clients, config, host
 * adapters) and never construct SDK clients themselves.
 */
export class AgentCatalogService {
  private readonly client: AgentsAppSdkClient;

  constructor(client: AgentsAppSdkClient) {
    this.client = client;
  }

  async listAgents(pageSize: number): Promise<AgentsCatalogListItem[]> {
    if (pageSize <= 0) {
      throw new Error('pageSize must be a positive integer');
    }
    // The ArkTS-adapted transport is not generated yet; the port is injected so
    // this service compiles unchanged once the adapter lands.
    return [];
  }

  baseUrl(): string {
    return this.client.baseUrl;
  }
}
`,
      },
      {
        path: 'src/main/ets/presentation/viewModels/AgentsCatalogViewModel.ets',
        content: `import { AgentCatalogService } from '../../services/AgentCatalogService';
import { type AgentsCatalogScreenModel } from '../../models/AgentModels';

/**
 * Catalog view model. Owns UI state mapping; calls services only.
 */
export class AgentsCatalogViewModel {
  private readonly service: AgentCatalogService;
  private model: AgentsCatalogScreenModel = {
    items: [],
    loading: false,
    hasMore: false,
  };

  constructor(service: AgentCatalogService) {
    this.service = service;
  }

  state(): AgentsCatalogScreenModel {
    return this.model;
  }

  async load(pageSize: number): Promise<AgentsCatalogScreenModel> {
    this.model = { items: this.model.items, loading: true, hasMore: false };
    try {
      const items = await this.service.listAgents(pageSize);
      this.model = { items, loading: false, hasMore: items.length >= pageSize };
    } catch (error) {
      this.model = {
        items: [],
        loading: false,
        hasMore: false,
        errorMessage: error instanceof Error ? error.message : 'unknown-error',
      };
    }
    return this.model;
  }
}
`,
      },
      {
        path: 'src/main/ets/state/AgentsCatalogState.ets',
        content: `/**
 * Package-local state slice descriptor.
 *
 * State stays package-local unless declared as a public integration contract.
 */
export interface AgentsCatalogStateSlice {
  readonly selectedAgentId?: string;
  readonly lastLoadedProfileId?: string;
}

export const initialAgentsCatalogStateSlice: AgentsCatalogStateSlice = {};
`,
      },
      {
        path: 'src/main/ets/i18n/AgentsMessages.ets',
        content: `/**
 * Package-local locale fragments.
 *
 * Authority: \`I18N_SPEC.md\` section 6.1 — split by locale, domain, capability,
 * and screen fragment. Platform monolithic resources are assembled from these
 * fragments and must not be hand-authored as a whole-root catalog.
 */
export const agentsMessagesEnUs: Record<string, string> = {
  'agents.catalog.title': 'Agents',
  'agents.detail.title': 'Agent',
  'agents.create.title': 'Create Agent',
  'agents.conversation.title': 'Conversation',
  'agents.catalog.empty': 'No agents yet',
  'agents.catalog.loadFailed': 'Failed to load agents',
};

export const agentsMessagesZhCn: Record<string, string> = {
  'agents.catalog.title': '智能体',
  'agents.detail.title': '智能体详情',
  'agents.create.title': '创建智能体',
  'agents.conversation.title': '对话',
  'agents.catalog.empty': '暂无智能体',
  'agents.catalog.loadFailed': '智能体加载失败',
};
`,
      },
      {
        path: 'src/main/ets/pages/AgentsCatalogPage.ets',
        content: `import { EmptyState } from '@sdkwork/sdkwork-agents-harmony-mobile-commons';

/**
 * Route-level capability page. Root pages mount this; business UI lives here.
 */
@Component
export struct AgentsCatalogPage {
  @Prop title: string = 'Agents';

  build() {
    Column() {
      Text(this.title)
        .fontSize(20)
        .fontWeight(FontWeight.Bold)
        .margin({ bottom: 16 })
      EmptyState({ message: 'No agents yet' })
    }
    .width('100%')
    .height('100%')
    .padding(16)
  }
}
`,
      },
    ],
    indexContent: `/**
 * SDKWork Agents HarmonyOS mobile agents capability public integration boundary.
 */
export * from './routes/RouteContributions';
export * from './models/AgentModels';
export * from './services/AgentCatalogService';
export * from './presentation/viewModels/AgentsCatalogViewModel';
export * from './state/AgentsCatalogState';
export * from './i18n/AgentsMessages';
export * from './pages/AgentsCatalogPage';
`,
  });
}

// ---------------------------------------------------------------------------
// Docs, sdks, tests
// ---------------------------------------------------------------------------

function materializeDocsAndTests() {
  write('docs/README.md', `# Documentation Canon

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)

HarmonyOS-specific design notes, package taxonomy decisions, and host adapter
boundaries are recorded here as the root matures.
`);

  write('sdks/README.md', `# sdks/

This directory follows \`SDK_WORKSPACE_GENERATION_SPEC.md\`.

Harmony roots consume the application-owned generated app SDK from the
repository-level \`sdks/\` workspace. They must not contain hand-edited generated
output.

Current coverage of \`sdks/sdkwork-agents-app-sdk\`:

| Target | Workspace | State |
| --- | --- | --- |
| typescript | \`sdkwork-agents-app-sdk-typescript\` | materialized |
| flutter | \`sdkwork-agents-app-sdk-flutter\` | materialized |
| arkts | _none_ | not produced by the SDK generation chain yet |

Until an ArkTS target exists, the HarmonyOS root consumes the TypeScript SDK
facade through an adapted port declared in
\`packages/sdkwork-agents-harmony-mobile-core/src/main/ets/sdk/AgentsAppSdkClient.ets\`.
`);

  write('scripts/README.md', `# scripts/

HarmonyOS build/release helper scripts belong here once the DevEco
toolchain is available. Static checks currently run from the repository root:

\`\`\`bash
node ../sdkwork-specs/tools/check-apps-directory-index.mjs --root .
node ../sdkwork-specs/tools/check-frontend-composition.mjs --root .
node --test apps/sdkwork-agents-harmony-mobile/tests/harmony-surface-contract.test.mjs
\`\`\`
`);

  write('tests/harmony-surface-contract.test.mjs', `import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Static architecture contract for the SDKWork Agents HarmonyOS mobile root.
 *
 * Authority: \`HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md\` section 11
 * (root layout / package naming / root thinness / SDK boundary) and
 * \`APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md\`.
 */

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function mustExist(relativePath) {
  const absolute = path.join(root, relativePath);
  assert.ok(fs.existsSync(absolute), \`\${relativePath} must exist\`);
  return fs.readFileSync(absolute, "utf8");
}

function listFiles(relativePath) {
  const absolute = path.join(root, relativePath);
  if (!fs.existsSync(absolute)) return [];
  return fs.readdirSync(absolute, { withFileTypes: true }).flatMap((entry) => {
    const childRelative = path.join(relativePath, entry.name);
    if (entry.isDirectory()) {
      if (["node_modules", "build", "oh_modules", ".hvigor"].includes(entry.name)) return [];
      return listFiles(childRelative);
    }
    return [childRelative];
  });
}

// --- Root layout -----------------------------------------------------------

for (const requiredPath of [
  "AGENTS.md",
  "README.md",
  "sdkwork.app.config.json",
  "specs/component.spec.json",
  "oh-package.json5",
  "build-profile.json5",
  "hvigorfile.ts",
  ".sdkwork/README.md",
  ".sdkwork/skills/README.md",
  ".sdkwork/plugins/README.md",
  "etc/README.md",
  "etc/sdkwork.deployment.config.json",
  "AppScope/app.json5",
  "entry/oh-package.json5",
  "entry/build-profile.json5",
  "entry/src/main/module.json5",
  "entry/src/main/ets/entryability/EntryAbility.ets",
  "sdks/README.md",
  "scripts/README.md",
]) {
  mustExist(requiredPath);
}

// --- Package family --------------------------------------------------------

const expectedPackages = [
  "sdkwork-agents-harmony-mobile-core",
  "sdkwork-agents-harmony-mobile-commons",
  "sdkwork-agents-harmony-mobile-shell",
  "sdkwork-agents-harmony-mobile-host",
  "sdkwork-agents-harmony-mobile-agents",
];

for (const packageName of expectedPackages) {
  const packageRoot = \`packages/\${packageName}\`;
  mustExist(\`\${packageRoot}/oh-package.json5\`);
  mustExist(\`\${packageRoot}/build-profile.json5\`);
  mustExist(\`\${packageRoot}/src/main/module.json5\`);
  mustExist(\`\${packageRoot}/src/main/ets/Index.ets\`);
  const componentSpec = JSON.parse(mustExist(\`\${packageRoot}/specs/component.spec.json\`));
  assert.equal(
    componentSpec.component?.root,
    \`apps/sdkwork-agents-harmony-mobile/packages/\${packageName}\`,
    \`\${packageName} component spec root must use the canonical HarmonyOS package path\`,
  );
  assert.ok(
    fs.existsSync(path.join(root, packageRoot, "specs/component.spec.json")),
    \`\${packageName} must declare a component spec\`,
  );
}

// --- Component spec layer roles -------------------------------------------

const allowedLayerRoles = new Set([
  "contract",
  "frontend-core",
  "frontend-shell",
  "frontend-feature",
  "frontend-commons",
  "frontend-host",
  "backend-route",
  "backend-service",
  "backend-domain",
  "backend-repository",
  "backend-provider",
  "runtime-api-server",
  "runtime-service-host",
  "runtime-composition",
  "runtime-gateway",
  "runtime-native-host",
  "sdk-facade",
  "sdk-generated",
  "tooling",
]);

for (const packageName of expectedPackages) {
  const componentSpec = JSON.parse(
    mustExist(\`packages/\${packageName}/specs/component.spec.json\`),
  );
  const layerRole = componentSpec.contracts?.layerRole;
  if (layerRole !== undefined) {
    assert.ok(
      allowedLayerRoles.has(layerRole),
      \`\${packageName} contracts.layerRole \${JSON.stringify(layerRole)} is not an allowed composable layer role\`,
    );
  }
}

// --- Root thinness ---------------------------------------------------------

const entryFiles = listFiles("entry/src/main/ets");
const businessOwnedEntryFiles = entryFiles.filter((filePath) => {
  const normalized = filePath.replaceAll("\\\\", "/");
  return /\\/pages\\/(?!Index\\.ets|__generated__)/u.test(normalized);
});
assert.deepEqual(
  businessOwnedEntryFiles,
  [],
  "root entry/ must stay thin: business pages belong in capability packages",
);

// --- Config -----------------------------------------------------------------

for (const deploymentProfile of ["standalone", "cloud"]) {
  for (const environment of ["development", "test", "staging", "production"]) {
    const profileId = \`\${deploymentProfile}.\${environment}\`;
    const runtimeConfig = JSON.parse(
      mustExist(\`config/app/runtime-env.\${profileId}.json\`),
    );
    assert.equal(runtimeConfig.profileId, profileId, \`\${profileId} must declare its profileId\`);
    assert.equal(
      runtimeConfig.deploymentProfile,
      deploymentProfile,
      \`\${profileId} must declare deploymentProfile=\${deploymentProfile}\`,
    );
    assert.equal(runtimeConfig.environment, environment, \`\${profileId} must declare environment=\${environment}\`);
    assert.equal(
      runtimeConfig.runtimeTarget,
      "harmony-native",
      \`\${profileId} must declare runtimeTarget=harmony-native\`,
    );
  }
}

// --- Host config must stay secret-free -------------------------------------

const hostConfigFiles = listFiles("config/host");
assert.ok(hostConfigFiles.length > 0, "config/host must contain checked-in templates");
const secretPattern = /(signingPrivateKey|privateKey|refreshToken|apiKey|databaseUrl|password)\\s*[:=]\\s*["'][^"'<]/iu;
for (const hostFile of hostConfigFiles) {
  const content = mustExist(hostFile);
  assert.doesNotMatch(content, secretPattern, \`\${hostFile} must not contain secrets\`);
}

// --- App manifest ----------------------------------------------------------

const manifest = JSON.parse(mustExist("sdkwork.app.config.json"));
assert.equal(manifest.app?.appType, "APP_HARMONY", "Harmony root must declare appType APP_HARMONY");
assert.equal(manifest.runtime?.family, "mobile", "Harmony root must declare runtime.family mobile");
assert.equal(
  manifest.runtime?.framework,
  "harmony-native",
  "Harmony root must declare runtime.framework harmony-native",
);
assert.ok(
  manifest.publish?.platforms?.includes("APP_HARMONY"),
  "Harmony root must publish for APP_HARMONY",
);

// --- SDK boundary ----------------------------------------------------------

const featureSource = listFiles("packages/sdkwork-agents-harmony-mobile-agents/src")
  .filter((filePath) => filePath.endsWith(".ets"))
  .map((filePath) => mustExist(filePath))
  .join("\\n");
assert.doesNotMatch(
  featureSource,
  /@ohos\\.net\\.http|http\\.createHttp|fetch\\(/u,
  "capability packages must not perform raw HTTP transport",
);

console.log("harmony surface contract passed.");
`);
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

materializeWorkspaceFiles();
materializeConfig();
materializeEtc();
materializeEntry();
materializePackages();
materializeDocsAndTests();

writeJson('specs/component.spec.json', rootComponentSpec());

if (!fs.existsSync(path.join(appRoot, 'docs/README.md'))) {
  throw new Error('docs/README.md failed to materialize');
}

console.log(`materialize-harmony-app-surface: apps/${appRootName}`);
console.log(`  created: ${created.length}`);
console.log(`  skipped (already present): ${skipped.length}`);
if (skipped.length > 0) {
  for (const relativePath of skipped) {
    console.log(`    - ${relativePath}`);
  }
}
