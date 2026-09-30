import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';
import { countAgentOpenApiOperations } from '../sdks/_shared/agent-sdk-ownership.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const openOpenApi = fs.readFileSync(
  path.join(
    root,
    'crates/sdkwork-intelligence-agents-service/specs/openapi/agents-open-api.openapi.yaml',
  ),
  'utf8',
);
const openSdkAi = fs.readFileSync(
  path.join(
    root,
    'sdks/sdkwork-agents-sdk/sdkwork-agents-sdk-typescript/generated/server-openapi/src/api/ai.ts',
  ),
  'utf8',
);
const openSdkGeneratorManifest = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      'sdks/sdkwork-agents-sdk/sdkwork-agents-sdk-typescript/generated/server-openapi/.sdkwork/sdkwork-generator-manifest.json',
    ),
    'utf8',
  ),
);
const appSdkAi = fs.readFileSync(
  path.join(
    root,
    'sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/generated/server-openapi/src/api/ai.ts',
  ),
  'utf8',
);
const appSdkFlutterAi = fs.readFileSync(
  path.join(
    root,
    'sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-flutter/generated/server-openapi/lib/src/api/ai.dart',
  ),
  'utf8',
);
const serviceHttp = fs.readFileSync(
  path.join(root, 'crates/sdkwork-intelligence-agents-service/src/http.rs'),
  'utf8',
);

assert.equal(countAgentOpenApiOperations(openOpenApi), 56);
assert.equal(openSdkGeneratorManifest.sdk?.sdkType, 'custom');
assert.equal(openSdkGeneratorManifest.sdk?.name, 'sdkwork-agents-sdk');
assert.equal((openSdkAi.match(/async \w+\(/g) ?? []).length, 56);
assert.doesNotMatch(openOpenApi, /\/agent\/v3\/api\/ai\/agents\/\{agentId\}\/restore:/);
assert.doesNotMatch(openOpenApi, /\/agent\/v3\/api\/ai\/agent_engines:/);
assert.doesNotMatch(openOpenApi, /\/agent\/v3\/api\/ai\/mcp_servers:/);

for (const forbidden of ['/agent_engines', '/mcp_servers']) {
  assert.doesNotMatch(
    openSdkAi,
    new RegExp(forbidden.replace('/', '\\/')),
    `open SDK ai.ts must not reference app-only path ${forbidden}`,
  );
}
const openSdkAgentsApi = extractClassBlock(openSdkAi, 'AiAgentsApi');
assert.doesNotMatch(openSdkAgentsApi, /\basync restore\(/);
assert.doesNotMatch(openSdkAi, /AiAgentsAgentEnginesApi|AiAgentsMcpServersApi/);
assert.doesNotMatch(openSdkAi, /AiAgentsProjectsApi|AiAgentsProjectCompositionSlotsApi/);
assert.doesNotMatch(openSdkAi, /AiAgentsSessionUserStatesApi|AiAgentsItemFeedbackApi/);

for (const className of ['AiAgentsProjectsApi', 'AiAgentsWorkspacesApi']) {
  const classBlock = extractClassBlock(appSdkAi, className);
  assert.match(
    classBlock,
    /\{ name: 'include_deleted', value: params\?\.includeDeleted,/,
    `${className} must serialize includeDeleted as include_deleted`,
  );
  assert.doesNotMatch(
    classBlock,
    /\{ name: 'includeDeleted',/,
    `${className} must not emit the camelCase query parameter`,
  );
}

for (const methodName of ['agentsProjectsList', 'agentsWorkspacesList']) {
  const methodBlock = extractDartMethodBlock(appSdkFlutterAi, methodName);
  assert.match(
    methodBlock,
    /QueryParameterSpec\('include_deleted', includeDeleted,/,
    `${methodName} must serialize includeDeleted as include_deleted`,
  );
  assert.doesNotMatch(
    methodBlock,
    /QueryParameterSpec\('includeDeleted',/,
    `${methodName} must not emit the camelCase query parameter`,
  );
}

assert.doesNotMatch(
  serviceHttp.match(/pub fn build_open_routes[\s\S]*?^}/m)?.[0] ?? serviceHttp,
  /\/agent\/v3\/api\/ai\/agents\/\{agentId\}\/restore/,
);

const openApiContracts = [
  ['app', 'agents-app-api.openapi.yaml', 127],
  ['backend', 'agents-backend-api.openapi.yaml', 60],
  ['open', 'agents-open-api.openapi.yaml', 56],
];
const forbiddenScopeFields = new Set(['tenantId', 'organizationId', 'ownerUserId']);
const forbiddenScopeParameters = new Set([
  'TenantId',
  'TenantIdQuery',
  'OrganizationId',
  'OwnerUserId',
]);

for (const [surface, basename, expectedOperationCount] of openApiContracts) {
  const source = fs.readFileSync(
    path.join(
      root,
      'crates/sdkwork-intelligence-agents-service/specs/openapi',
      basename,
    ),
    'utf8',
  );
  const contract = YAML.parse(source);
  const schemas = contract.components?.schemas ?? {};

  if (surface === 'open') {
    assert.deepEqual(contract.components?.securitySchemes, {
      ApiKey: { type: 'apiKey', in: 'header', name: 'X-API-Key' },
    });
  }

  assert.equal(
    countAgentOpenApiOperations(source),
    expectedOperationCount,
    `${surface} operation inventory drifted`,
  );
  assertNoUnreachableComponents(contract, surface);
  assert.equal(
    schemas.AgentCompositionSlotCreateData,
    undefined,
    `${surface} must not expose a nested composition-slot create data schema`,
  );
  assert.equal(
    schemas.UpdateAgentCompositionSlotData,
    undefined,
    `${surface} must not expose a nested composition-slot update data schema`,
  );
  assert.equal(
    schemas.CreateAgentCompositionSlotRequest?.properties?.data,
    undefined,
    `${surface} composition-slot create request must be flat`,
  );
  assert.equal(
    schemas.UpdateAgentCompositionSlotRequest?.properties?.data,
    undefined,
    `${surface} composition-slot update request must be flat`,
  );
  assert.equal(
    schemas.AgentCompositionSlotRecord?.properties?.priority?.type,
    'integer',
    `${surface} composition-slot priority must use its native integer type`,
  );

  if (surface === 'app') {
    for (const [pathName, operationId] of [
      ['/app/v3/api/ai/projects', 'agents.projects.list'],
      ['/app/v3/api/ai/workspaces', 'agents.workspaces.list'],
    ]) {
      const parameters = (contract.paths?.[pathName]?.get?.parameters ?? []).map(
        (parameter) =>
          parameter.$ref
            ? contract.components?.parameters?.[parameter.$ref.split('/').at(-1)]
            : parameter,
      );
      assert.ok(
        parameters.some((parameter) => parameter?.name === 'include_deleted'),
        `${operationId} must declare the include_deleted wire parameter`,
      );
      assert.ok(
        parameters.every((parameter) => parameter?.name !== 'includeDeleted'),
        `${operationId} must not declare a camelCase query parameter`,
      );
    }
  }

  const sessionRequest = schemas.CreateAgentSessionRequest;
  for (const required of [
    'sessionKind',
    'entrySurface',
    'idempotencyKey',
    'payloadHash',
    'requestedAt',
  ]) {
    assert.ok(
      sessionRequest?.required?.includes(required),
      `${surface} CreateAgentSessionRequest must require ${required}`,
    );
  }

  const turnStreamOperation = Object.values(contract.paths ?? {})
    .flatMap((pathItem) => Object.values(pathItem ?? {}))
    .find((operation) => operation?.operationId === 'agents.turns.stream');
  assert.equal(
    turnStreamOperation?.responses?.['200']?.content?.['text/event-stream']?.schema?.$ref,
    '#/components/schemas/AgentTurnStreamEvent',
    `${surface} turn stream must use the typed event schema`,
  );
  assert.deepEqual(
    schemas.AgentTurnStreamEvent?.required,
    ['eventType'],
    `${surface} turn stream event must require eventType`,
  );
  assert.deepEqual(
    schemas.AgentTurnStreamEvent?.properties?.eventType?.enum,
    ['event', 'delta', 'completion'],
    `${surface} turn stream event types drifted`,
  );
  assert.equal(
    schemas.AgentTurnStreamEvent?.properties?.response?.$ref,
    '#/components/schemas/AgentTurnExecutionResponse',
    `${surface} completion stream event must carry the canonical execution response`,
  );

  for (const [pathName, pathItem] of Object.entries(contract.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (!['get', 'post', 'put', 'patch', 'delete'].includes(method)) continue;
      if (surface === 'open') {
        assert.deepEqual(
          operation.security,
          [{ ApiKey: [] }],
          `open ${method.toUpperCase()} ${pathName} must require X-API-Key`,
        );
      }
      const parameters = [...(pathItem.parameters ?? []), ...(operation.parameters ?? [])];
      for (const parameter of parameters) {
        const name = parameter.$ref?.split('/').at(-1) ?? parameter.name;
        assert.ok(
          !forbiddenScopeParameters.has(name),
          `${surface} ${method.toUpperCase()} ${pathName} must derive ${name} from request context`,
        );
      }
    }
  }

  for (const schemaName of collectRequestSchemas(contract)) {
    const schema = schemas[schemaName];
    for (const field of forbiddenScopeFields) {
      assert.equal(
        schema?.properties?.[field],
        undefined,
        `${surface} request schema ${schemaName} must not expose ${field}`,
      );
    }
  }
}

function collectRequestSchemas(contract) {
  const collected = new Set();
  const collectRefs = (value) => {
    if (!value || typeof value !== 'object') return;
    if (typeof value.$ref === 'string' && value.$ref.startsWith('#/components/schemas/')) {
      collected.add(value.$ref.split('/').at(-1));
    }
    for (const nested of Object.values(value)) collectRefs(nested);
  };
  for (const pathItem of Object.values(contract.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (['post', 'put', 'patch', 'delete'].includes(method)) collectRefs(operation.requestBody);
    }
  }
  let expanded = true;
  while (expanded) {
    expanded = false;
    for (const schemaName of [...collected]) {
      const before = collected.size;
      collectRefs(contract.components?.schemas?.[schemaName]);
      expanded ||= collected.size !== before;
    }
  }
  return collected;
}

function assertNoUnreachableComponents(contract, surface) {
  const components = contract.components ?? {};
  const reachable = new Map(
    Object.keys(components).map((kind) => [kind, new Set()]),
  );
  const visit = (value) => {
    if (!value || typeof value !== 'object') return;
    for (const nested of Object.values(value)) {
      if (typeof nested === 'string' && nested.startsWith('#/components/')) {
        const [, , kind, name] = nested.split('/');
        const names = reachable.get(kind);
        if (names && !names.has(name)) {
          names.add(name);
          visit(components[kind]?.[name]);
        }
      } else {
        visit(nested);
      }
    }
  };
  visit(contract.paths);

  for (const [kind, entries] of Object.entries(components)) {
    if (kind === 'securitySchemes') continue;
    const unused = Object.keys(entries ?? {}).filter(
      (name) => !reachable.get(kind)?.has(name),
    );
    assert.deepEqual(
      unused,
      [],
      `${surface} components.${kind} contains unreachable definitions`,
    );
  }

  const usedSecuritySchemes = new Set();
  const collectSecurity = (security) => {
    for (const requirement of security ?? []) {
      for (const name of Object.keys(requirement)) usedSecuritySchemes.add(name);
    }
  };
  collectSecurity(contract.security);
  for (const pathItem of Object.values(contract.paths ?? {})) {
    collectSecurity(pathItem.security);
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (['get', 'post', 'put', 'patch', 'delete'].includes(method)) {
        collectSecurity(operation.security);
      }
    }
  }
  const unusedSecuritySchemes = Object.keys(components.securitySchemes ?? {}).filter(
    (name) => !usedSecuritySchemes.has(name),
  );
  assert.deepEqual(
    unusedSecuritySchemes,
    [],
    `${surface} components.securitySchemes contains unreachable definitions`,
  );
}

function extractClassBlock(source, className) {
  const match = source.match(
    new RegExp(`export class ${className}[\\s\\S]*?^}\\r?\\n`, 'm'),
  );
  assert.ok(match, `generated open SDK must expose ${className}`);
  return match[0];
}

function extractDartMethodBlock(source, methodName) {
  const match = source.match(
    new RegExp(`^  Future<[^\\n]+> ${methodName}\\([^\\n]*\\) async \\{[\\s\\S]*?^  \\}`, 'm'),
  );
  assert.ok(match, `generated app Flutter SDK must expose ${methodName}`);
  return match[0];
}

console.log('agents open sdk surface contract passed.');
