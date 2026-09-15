import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import {
  AGENTS_SDK_FAMILIES,
  resolveAgentsSdkFamily,
  resolveAgentsSdkLanguageTargets
} from './_shared/agents-sdk-families.mjs';
import { syncAgentSdkOwnershipWorkspace } from './_shared/agent-sdk-ownership.mjs';
import {
  SDKWORK_SDKGEN_STANDARD,
  resolveSdkgenEntrypoint
} from './_shared/sdkgen-standard.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = parseArgs(process.argv.slice(2));
const mode = args.mode ?? 'dry-run';
if (!['dry-run', 'apply'].includes(mode)) {
  throw new Error(`--mode must be dry-run or apply, received: ${mode}`);
}

const requestedFamily = args.family;
const families = requestedFamily
  ? [resolveAgentsSdkFamily(args.family)]
  : AGENTS_SDK_FAMILIES;
const sdkgenPath = resolveSdkgenEntrypoint();
const sdkgenReportPath = toReportPath(sdkgenPath);

if (!fs.existsSync(sdkgenPath)) {
  throw new Error(
    `sdkgen entrypoint not found: ${sdkgenPath}. Set ${SDKWORK_SDKGEN_STANDARD.envOverride} only to another sdkwork-sdk-generator entrypoint.`
  );
}

runNodeScript(path.join(root, 'sdks', 'materialize-agent-v3-openapi-boundaries.mjs'), []);

const report = {
  schemaVersion: 1,
  app: 'agents',
  mode,
  standardProfile: SDKWORK_SDKGEN_STANDARD.standardProfile,
  sdkgenPath: sdkgenReportPath,
  startedAt: new Date().toISOString(),
  families: []
};

for (const family of families) {
  const input = path.join(
    root,
    'sdks',
    family.familyDir,
    'openapi',
    `${family.authority}.sdkgen.yaml`
  );
  const languageTargets = resolveAgentsSdkLanguageTargets(family);

  if (family.externalSdkgenProfileSupported === false) {
    report.families.push({
      key: family.key,
      familyDir: family.familyDir,
      authority: family.authority,
      input: toReportPath(input),
      sdkName: family.sdkName,
      sdkType: family.sdkType,
      sdkSurface: family.sdkSurface,
      packageName: family.packageName,
      apiPrefix: family.apiPrefix,
      generated: false,
      skipped: true,
      skipReason:
        family.externalSdkgenProfileGap ??
        'The canonical SDK generator profile is not available for this family.',
      languages: languageTargets.map((target) => ({
        language: target.language,
        workspace: target.workspace,
      })),
    });
    continue;
  }

  const languageReports = [];
  for (const target of languageTargets) {
    const output = path.join(
      root,
      'sdks',
      family.familyDir,
      target.workspace,
      'generated',
      'server-openapi'
    );
    fs.mkdirSync(output, { recursive: true });
    const baseArgs = buildGeneratorArgs(family, target, input, output);
    const dryRun = runNodeForJson([
      ...baseArgs,
      '--fixed-sdk-version',
      '0.1.0',
      '--dry-run',
      '--json'
    ]);
    const plannedChanges = Boolean(dryRun.hasChanges);
    const languageReport = {
      language: target.language,
      workspace: target.workspace,
      output: toReportPath(output),
      packageName: target.packageName,
      version: dryRun.sdk?.version ?? '0.1.0',
      fingerprint: dryRun.changeFingerprint,
      hasChanges: plannedChanges,
      riskLevel: dryRun.executionDecision?.riskLevel ?? 'unknown',
      generated: false
    };

    if (mode === 'apply') {
      if (!dryRun.changeFingerprint && plannedChanges) {
        throw new Error(
          `${family.familyDir}/${target.language} dry-run did not return a change fingerprint`
        );
      }
      if (plannedChanges) {
        runNodeScript(sdkgenPath, [
          ...baseArgs.slice(1),
          '--fixed-sdk-version',
          languageReport.version,
          '--expected-change-fingerprint',
          languageReport.fingerprint,
          '--license',
          'MIT'
        ]);
      }
      languageReport.generated = plannedChanges;
      languageReport.hasChanges = false;
    }
    languageReports.push(languageReport);
  }

  const typescriptReport = languageReports.find(
    (languageReport) => languageReport.language === 'typescript'
  );
  report.families.push({
    key: family.key,
    familyDir: family.familyDir,
    authority: family.authority,
    input: toReportPath(input),
    output: typescriptReport?.output,
    sdkName: family.sdkName,
    sdkType: family.sdkType,
    sdkOwner: family.sdkOwner,
    packageName: family.packageName,
    apiPrefix: family.apiPrefix,
    sdkDependencies: family.sdkDependencies,
    version: typescriptReport?.version ?? '0.1.0',
    fingerprint: typescriptReport?.fingerprint,
    fingerprints: Object.fromEntries(
      languageReports.map((languageReport) => [
        languageReport.language,
        languageReport.fingerprint,
      ])
    ),
    hasChanges: languageReports.some((languageReport) => languageReport.hasChanges),
    riskLevel: highestRiskLevel(languageReports),
    generated: languageReports.some((languageReport) => languageReport.generated),
    languages: languageReports
  });
}

syncAgentSdkOwnershipWorkspace(root, AGENTS_SDK_FAMILIES);
report.finishedAt = new Date().toISOString();
writeJson(path.join(root, 'sdks', '.sdkgen-agent-workspace-report.json'), report);
console.log(JSON.stringify(report, null, 2));

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--mode') {
      parsed.mode = argv[++index];
    } else if (value === '--family') {
      parsed.family = argv[++index];
    } else if (value === '--help' || value === '-h') {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${value}`);
    }
  }
  return parsed;
}

function printHelpAndExit() {
  console.log(`Usage: node sdks/workspace-agent-sdkgen.mjs [--mode dry-run|apply] [--family open|app|backend]

Generates SDKWork agents SDK families with --standard-profile ${SDKWORK_SDKGEN_STANDARD.standardProfile}.
`);
  process.exit(0);
}

function buildGeneratorArgs(family, target, input, output) {
  return [
    sdkgenPath,
    'generate',
    '-i',
    input,
    '-o',
    output,
    '-n',
    family.sdkName,
    '-t',
    family.sdkType,
    '-l',
    target.language,
    '--base-url',
    'http://localhost:8080',
    '--api-prefix',
    family.apiPrefix,
    '--package-name',
    target.packageName,
    ...(target.npmPackageName
      ? ['--npm-package-name', target.npmPackageName]
      : []),
    ...(target.language === 'typescript'
      ? [
          '--common-package',
          `${SDKWORK_SDKGEN_STANDARD.typescriptCommonPackage.name}@${SDKWORK_SDKGEN_STANDARD.typescriptCommonPackage.version}`,
        ]
      : []),
    '--sdk-root',
    path.join(root, 'sdks', family.familyDir),
    '--sdk-name',
    family.sdkName,
    '--standard-profile',
    SDKWORK_SDKGEN_STANDARD.standardProfile,
    '--no-sync-published-version'
  ];
}

function highestRiskLevel(languageReports) {
  const rank = new Map([
    ['unknown', 0],
    ['low', 1],
    ['medium', 2],
    ['high', 3]
  ]);
  return languageReports.reduce(
    (highest, report) =>
      (rank.get(report.riskLevel) ?? 0) > (rank.get(highest) ?? 0)
        ? report.riskLevel
        : highest,
    'unknown'
  );
}

function runNodeForJson(nodeArgs) {
  const result = spawnSync('node', nodeArgs, {
    cwd: root,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe']
  });
  if (result.status !== 0) {
    throw new Error(
      `Command failed: node ${nodeArgs.join(' ')}\n${result.stdout}\n${result.stderr}`
    );
  }
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`Failed to parse sdkgen JSON output: ${error.message}\n${result.stdout}`);
  }
}

function runNodeScript(script, scriptArgs) {
  const result = spawnSync('node', [script, ...scriptArgs], {
    cwd: root,
    encoding: 'utf8',
    stdio: 'inherit'
  });
  if (result.status !== 0) {
    throw new Error(`Command failed: node ${script} ${scriptArgs.join(' ')}`);
  }
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function toReportPath(filePath) {
  const absolute = path.resolve(root, filePath);
  const withinRepo = path.relative(root, absolute);
  if (withinRepo && !withinRepo.startsWith('..') && !path.isAbsolute(withinRepo)) {
    return normalizeReportPath(withinRepo);
  }
  // Sibling repositories (`sdkwork-sdk-generator` among them) live outside this
  // repository but inside the same relocatable checkout root. Recording the raw
  // absolute path there froze the generating machine's drive letter into a
  // committed report; record it relative to the checkout root instead, which is
  // the parent directory every `sdkwork-*` repository shares.
  const checkoutRoot = path.dirname(root);
  const withinCheckout = path.relative(checkoutRoot, absolute);
  if (withinCheckout && !withinCheckout.startsWith('..') && !path.isAbsolute(withinCheckout)) {
    return normalizeReportPath(withinCheckout);
  }
  return normalizeReportPath(filePath);
}

function normalizeReportPath(filePath) {
  return String(filePath).replace(/\\/g, '/');
}
