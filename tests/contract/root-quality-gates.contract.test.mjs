import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const requiredCheckScripts = [
  "check:api-operation-patterns",
  "check:pagination",
  "check:app-sdk-consumer-imports",
  "check:agent-sdk-workspace",
  "check:route-path-collisions",
  "check:apps-directory-index",
  "check:component-port-bindings",
  "check:frontend-service-identity",
  "check:frontend-composition",
  "check:permission-composition",
  "check:composition-resolver",
  "check:rust-backend-composition",
  "check:production-security",
  "check:source-config-standard",
  "deploy:validate:cloud",
];

test("root check command includes production readiness quality gates", () => {
  const packageJson = JSON.parse(
    readFileSync(path.join(repoRoot, "package.json"), "utf8"),
  );
  const scripts = packageJson.scripts ?? {};
  assert.equal(scripts.check, 'pnpm exec sdkwork-app check');
  const checkCommand = scripts['_sdkwork:check'] ?? '';

  for (const script of requiredCheckScripts) {
    assert.ok(scripts[script], `package.json must expose ${script}`);
    assert.match(
      checkCommand,
      new RegExp(`pnpm (run )?${script.replaceAll(":", "\\:")}\\b`),
      `pnpm check must run ${script}`,
    );
  }
});

const launchGateDocumentationFiles = [
  "docs/runbooks/pre-launch-verification.md",
  "docs/architecture/tech/TECH_ARCHITECTURE.md",
  "docs/product/prd/PRD.md",
];

test("launch documentation includes cloud deployment profile validation gate", () => {
  for (const relativePath of launchGateDocumentationFiles) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    assert.match(
      content,
      /deploy:validate:cloud/u,
      `${relativePath} must document deploy:validate:cloud as a cloud launch gate`,
    );
  }
});

const rootVerificationContractFiles = [
  "specs/README.md",
  "specs/component.spec.json",
];

test("root specs expose cloud deployment validation as a production verification command", () => {
  for (const relativePath of rootVerificationContractFiles) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    assert.match(
      content,
      /deploy:validate:cloud/u,
      `${relativePath} must expose deploy:validate:cloud in root verification evidence`,
    );
  }
});

const requiredContractTestFiles = [
  "tests/contract/production-security.contract.test.mjs",
  "tests/contract/frontend-service-identity.contract.test.mjs",
];

test("contract check command includes production security contracts", () => {
  const packageJson = JSON.parse(
    readFileSync(path.join(repoRoot, "package.json"), "utf8"),
  );
  const scripts = packageJson.scripts ?? {};
  const checkContractsCommand = scripts["check:contracts"] ?? "";

  for (const relativePath of requiredContractTestFiles) {
    assert.match(
      checkContractsCommand,
      new RegExp(relativePath.replaceAll("/", "[\\\\/]")),
      `pnpm check:contracts must run ${relativePath}`,
    );
  }
});

test("root workflow scripts do not expose migration-only IM synchronization tools", () => {
  const packageJson = JSON.parse(
    readFileSync(path.join(repoRoot, "package.json"), "utf8"),
  );
  const scripts = packageJson.scripts ?? {};

  for (const [scriptName, command] of Object.entries(scripts)) {
    assert.doesNotMatch(
      `${scriptName} ${command}`,
      /\b(?:sdkwork-im|im-agent|im-pc-agent|migrate-im|retarget-im|sync-im|run-im)\b/,
      `${scriptName} must not expose migration-only sdkwork-im synchronization in the agents release surface`,
    );
  }

  const migrationOnlyScripts = readdirSync(path.join(repoRoot, "scripts")).filter((fileName) =>
    /(?:^|-)im(?:-|_).*agent|agent.*(?:^|-)im(?:-|_)/.test(fileName),
  );
  assert.deepEqual(
    migrationOnlyScripts,
    [],
    "migration-only sdkwork-im agent synchronization scripts must be removed from the agents release root",
  );
});

test("server release artifact includes immutable runtime and deployment assets", () => {
  const workflow = JSON.parse(
    readFileSync(path.join(repoRoot, "agent.workflow.json"), "utf8"),
  );
  // Framework profile validation maps `server` to runtimeTarget `server`
  // only; the server release bundle is the container- and bare-metal-ready
  // artifact that must carry every immutable runtime and deployment asset.
  const serverTarget = workflow.targets.find(
    (target) => target.profile === "server" && target.runtimeTarget === "server",
  );

  assert.ok(serverTarget, "workflow must declare a server target");
  for (const requiredGlob of [
    "database/**",
    "etc/topology/*.production.env",
    "deployments/**",
    "agent.app.config.json",
  ]) {
    assert.ok(
      serverTarget.outputGlobs.includes(requiredGlob),
      `server artifact must include ${requiredGlob}`,
    );
  }
  assert.ok(
    serverTarget.outputGlobs.some((glob) =>
      glob.startsWith("target/release/sdkwork-api-agents-standalone-gateway"),
    ),
    "server artifact must include the release gateway binary",
  );
});

const launchCanonFiles = [
  "apps/README.md",
  "docs/product/prd/PRD.md",
  "docs/architecture/tech/TECH_ARCHITECTURE.md",
  "docs/architecture/tech/TECH-api-specification.md",
  "docs/guides/operator/README.md",
  "docs/runbooks/pre-launch-verification.md",
  "crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md",
  "crates/sdkwork-intelligence-agents-service/src/api.rs",
  "specs/README.md",
  "specs/AGENTS_KERNEL_BOUNDARY_SPEC.md",
  "specs/AGENTS_KERNEL_SPI_GAP_ANALYSIS.md",
  "specs/AGENTS_PROVIDER_TAXONOMY_SPEC.md",
  "specs/agents-birdcoder-alignment.spec.json",
  "scripts/client-surface-readiness-contract.test.mjs",
];

const forbiddenLaunchScopePatterns = [
  {
    pattern: /\bTBD\b/i,
    guidance: "replace placeholder TBD wording with explicit ownership, scope, or entry criteria",
  },
  {
    pattern: /\bbinding TBD\b/i,
    guidance: "state the provider binding entry criteria instead of binding TBD wording",
  },
  {
    pattern: /\bfederation TBD\b/i,
    guidance: "state the federation entry criteria instead of federation TBD wording",
  },
  {
    pattern: /\bpost-GA\b/i,
    guidance: "use current GA scope or non-GA scope ownership instead of post-GA wording",
  },
  {
    pattern: /\bpost_GA\b/i,
    guidance: "use current GA scope or non-GA scope ownership instead of post_GA naming",
  },
  {
    pattern: /\bpost-launch\b/i,
    guidance: "use GA scope or non-GA scope ownership instead of post-launch wording",
  },
  {
    pattern: /\bpost_launch\b/i,
    guidance: "use GA scope or non-GA scope ownership instead of post_launch naming",
  },
  {
    pattern: /\bcloud\s+split\b/i,
    guidance: "use canonical topology profile ids such as cloud.production instead of retired cloud split wording",
  },
  {
    pattern: /\bstandalone\s+unified\b/i,
    guidance: "use canonical topology profile ids such as standalone.production instead of retired standalone unified wording",
  },
  {
    pattern: /\bsplit\s+services\b/i,
    guidance: "use canonical topology profile ids instead of retired split services wording",
  },
  {
    pattern: /\bunified\s+process\b/i,
    guidance: "use canonical topology profile ids instead of retired unified process wording",
  },
  {
    pattern: /\bCommercial GA blocker\b/i,
    guidance: "express release exclusions as scoped ownership and entry criteria",
  },
  {
    pattern: /\bnot blockers?\b/i,
    guidance: "do not label active launch gaps as non-blockers",
  },
  {
    pattern: /\bpending-dart-sdk\b/i,
    guidance: "describe Flutter as out of GA scope until an owned Dart SDK is available",
  },
  {
    pattern: /\bH5 fallback\b/i,
    guidance: "describe mini-program WebView coverage as an explicit editor bridge",
  },
  {
    pattern: /\bupload deferred\b/i,
    guidance: "describe upload as out of current product scope unless Drive Uploader is wired",
  },
  {
    pattern: /非上线阻塞|非阻塞上线|上线非阻塞/u,
    guidance: "express maintainability work as owned scoped engineering work with verification entry criteria instead of a launch-blocker disclaimer",
  },
];

test("launch canon files use explicit GA scope instead of deferred debt wording", () => {
  for (const relativePath of launchCanonFiles) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    for (const { pattern, guidance } of forbiddenLaunchScopePatterns) {
      assert.doesNotMatch(
        content,
        pattern,
        `${relativePath} must ${guidance}`,
      );
    }
  }
});

function collectRuntimeDatabaseFiles(directory, results = []) {
  const skipDirectories = new Set([
    ".git",
    ".runtime",
    "node_modules",
    "target",
    "dist",
  ]);

  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (!skipDirectories.has(entry.name)) {
        collectRuntimeDatabaseFiles(entryPath, results);
      }
      continue;
    }
    if (/\.(?:db|sqlite|sqlite3)$/u.test(entry.name)) {
      results.push(path.relative(repoRoot, entryPath).replaceAll("\\", "/"));
    }
  }

  return results;
}

function collectMarkdownFiles(directory, results = []) {
  const skipDirectories = new Set([
    ".git",
    ".runtime",
    "node_modules",
    "target",
    "dist",
  ]);

  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (!skipDirectories.has(entry.name)) {
        collectMarkdownFiles(entryPath, results);
      }
      continue;
    }
    if (entry.name.endsWith(".md")) {
      results.push(entryPath);
    }
  }

  return results;
}

test("cross-repository cargo examples declare explicit manifest paths", () => {
  const markdownRoots = [
    path.join(repoRoot, "docs"),
    path.join(repoRoot, "specs"),
    path.join(repoRoot, "database"),
  ];
  const siblingPackageManifestRequirements = [
    {
      packagePattern: /\bsdkwork-birdcoder-/u,
      manifestPattern: /--manifest-path\s+\.{2}[\\/]+sdkwork-birdcoder[\\/]+Cargo\.toml/u,
    },
    {
      packagePattern: /\bsdkwork-agent-provider-/u,
      manifestPattern: /--manifest-path\s+\.{2}[\\/]+sdkwork-kernel[\\/]+Cargo\.toml/u,
    },
  ];

  for (const markdownFile of markdownRoots.flatMap((root) => collectMarkdownFiles(root))) {
    const relativePath = path.relative(repoRoot, markdownFile).replaceAll("\\", "/");
    const lines = readFileSync(markdownFile, "utf8").split(/\r?\n/u);
    for (const [index, line] of lines.entries()) {
      if (!/^\s*(?:[-*]\s+)?cargo\s+test\s+/u.test(line)) {
        continue;
      }

      for (const { packagePattern, manifestPattern } of siblingPackageManifestRequirements) {
        if (!packagePattern.test(line)) {
          continue;
        }
        assert.match(
          line,
          manifestPattern,
          `${relativePath}:${index + 1} must include an explicit sibling workspace --manifest-path`,
        );
      }
    }
  }
});

test("source tree does not contain mutable runtime database files", () => {
  const runtimeDatabases = collectRuntimeDatabaseFiles(repoRoot);

  assert.deepEqual(
    runtimeDatabases,
    [],
    "runtime database files must be generated in .runtime/, temp dirs, or test fixtures built from SQL; do not track mutable database binaries",
  );
});

test("Cargo lock does not retain sqlx-postgres versions with known Rust future incompatibilities", () => {
  const cargoLock = readFileSync(path.join(repoRoot, "Cargo.lock"), "utf8");

  assert.doesNotMatch(
    cargoLock,
    /\[\[package\]\]\s+name = "sqlx-postgres"\s+version = "0\.8\.0"/u,
    "sqlx-postgres 0.8.0 triggers Rust future-incompat lints; update the Cargo-resolved SQLx patch line instead of retaining this lock entry",
  );
});

const sessionBridgeFiles = [
  "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src/session/session.ts",
  "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-core/src/session/session.ts",
];

function collectAppManifests() {
  const manifests = ["agent.app.config.json"];
  const appsRoot = path.join(repoRoot, "apps");
  for (const entry of readdirSync(appsRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) {
      continue;
    }
    const manifestPath = path.join("apps", entry.name, "agent.app.config.json");
    try {
      readFileSync(path.join(repoRoot, manifestPath), "utf8");
      manifests.push(manifestPath);
    } catch {
      // Non-application helper directories under apps/ are ignored here.
    }
  }
  return manifests;
}

function hasGeneratedPlaceholder(value) {
  if (Array.isArray(value)) {
    return value.some((item) => hasGeneratedPlaceholder(item));
  }
  if (value && typeof value === "object") {
    if (value.generatedPlaceholder === true) {
      return true;
    }
    return Object.values(value).some((item) => hasGeneratedPlaceholder(item));
  }
  return false;
}

test("pre-launch app manifests do not advertise stable release channels", () => {
  for (const relativePath of collectAppManifests()) {
    const manifest = JSON.parse(readFileSync(path.join(repoRoot, relativePath), "utf8"));
    const publishStatus = manifest.publish?.status;
    const release = manifest.release ?? {};
    const latestChannels = Object.keys(release.latest ?? {});
    const noteChannels = (release.notes ?? []).map((note) => note.channel).filter(Boolean);
    const usesGeneratedMedia = hasGeneratedPlaceholder(manifest.media);

    if (publishStatus === "BETA" || usesGeneratedMedia) {
      assert.notEqual(
        release.defaultChannel,
        "STABLE",
        `${relativePath} must not default to STABLE while publish.status is BETA or media is generated placeholder`,
      );
      assert.ok(
        !latestChannels.includes("STABLE"),
        `${relativePath} must not publish release.latest.STABLE before GA-ready media and release status`,
      );
      assert.ok(
        !noteChannels.includes("STABLE"),
        `${relativePath} must not publish STABLE release notes before GA-ready media and release status`,
      );
      assert.equal(
        release.defaultChannel,
        "BETA",
        `${relativePath} must use BETA as the default channel while publish.status is BETA`,
      );
      assert.deepEqual(
        latestChannels,
        ["BETA"],
        `${relativePath} must expose only release.latest.BETA while in BETA publish status`,
      );
    }
  }
});

test("frontend session bridges do not synthesize IAM runtime context defaults", () => {
  for (const relativePath of sessionBridgeFiles) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    assert.doesNotMatch(
      content,
      /environment:\s*\(\s*environment\s*\?\?\s*['"]dev['"]\s*\)/u,
      `${relativePath} must preserve token-derived IAM environment instead of defaulting it locally`,
    );
    assert.doesNotMatch(
      content,
      /deploymentMode:\s*\(\s*deploymentMode\s*\?\?\s*['"]saas['"]\s*\)/u,
      `${relativePath} must preserve IAM deploymentMode compatibility without defaulting it locally`,
    );
    assert.doesNotMatch(
      content,
      /authLevel:\s*\(\s*authLevel\s*\?\?\s*['"]password['"]\s*\)/u,
      `${relativePath} must preserve token-derived IAM authLevel instead of defaulting it locally`,
    );
  }
});

test("policy provider docs describe production IAM integration without placeholder wording", () => {
  const content = readFileSync(
    path.join(repoRoot, "crates/sdkwork-intelligence-agents-service/src/infrastructure.rs"),
    "utf8",
  );

  assert.doesNotMatch(
    content,
    /\bplaceholder\b|\bnot yet wired\b/u,
    "policy provider docs must describe current fail-closed and IAM-gated behavior, not historical placeholder integration",
  );
});

test("service in-process tests do not depend on dev-only policy bypass", () => {
  const testSources = [
    "crates/sdkwork-intelligence-agents-service/src/http.rs",
    "crates/sdkwork-intelligence-agents-service/src/application.rs",
  ];

  for (const relativePath of testSources) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    assert.doesNotMatch(
      content,
      /AllowAllPolicyProvider::allow\(/u,
      `${relativePath} tests must use IAM-gated policy providers instead of environment-sensitive dev bypass providers`,
    );
    assert.doesNotMatch(
      content,
      /SDKWORK_DEPLOYMENT_ENV|SDKWORK_AGENTS_DEV_AUTH_BYPASS|EnvVarRestore|env_test_lock/u,
      `${relativePath} tests must not mutate global auth/deployment environment variables`,
    );
    assert.match(
      content,
      /IamGatedPolicyProvider/u,
      `${relativePath} tests must exercise the production-equivalent IAM-gated policy path`,
    );
  }
});

test("service contract tests use production-equivalent policy providers", () => {
  const serviceContractSources = [
    "crates/sdkwork-intelligence-agents-service/tests/agent_business_service_contracts.rs",
    "crates/sdkwork-intelligence-agents-service/tests/http_axum_contracts.rs",
  ];

  for (const relativePath of serviceContractSources) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    assert.doesNotMatch(
      content,
      /\bAllowAllPolicyProvider\b|PolicyMode::Deny/u,
      `${relativePath} must not use allow-all policy providers for service contract coverage`,
    );
    assert.match(
      content,
      /IamGatedPolicyProvider/u,
      `${relativePath} must exercise the IAM-gated business policy provider on success paths`,
    );
    assert.match(
      content,
      /DenyAllPolicyProvider/u,
      `${relativePath} must use explicit deny-all providers for authorization failure paths`,
    );
    assert.match(
      content,
      /ai\.agents\.manage/u,
      `${relativePath} subjects must carry explicit agents IAM permission scopes`,
    );
  }
});

test("route web framework tests use IAM-gated business policy providers", () => {
  const routeTestSources = [
    "crates/sdkwork-routes-agents-app-api/tests/app_web_framework_routes.rs",
    "crates/sdkwork-routes-agents-backend-api/tests/backend_web_framework_routes.rs",
    "crates/sdkwork-routes-agents-open-api/tests/open_web_framework_routes.rs",
  ];

  for (const relativePath of routeTestSources) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    assert.doesNotMatch(
      content,
      /AllowAllPolicyProvider::allow\(/u,
      `${relativePath} must not bypass agents business policy with dev-only allow-all provider`,
    );
    assert.doesNotMatch(
      content,
      /SDKWORK_AGENTS_DEV_AUTH_BYPASS|SDKWORK_DEPLOYMENT_ENV/u,
      `${relativePath} must not depend on agents dev auth bypass deployment environment gates`,
    );
    assert.match(
      content,
      /IamGatedPolicyProvider/u,
      `${relativePath} must exercise the production-equivalent IAM-gated business policy provider`,
    );
    assert.match(
      content,
      /ai\.agents\.manage/u,
      `${relativePath} authenticated test credentials must carry explicit IAM permission scope`,
    );
  }
});
