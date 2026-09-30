# SDKWork Agents API And SDK Reference

- Version: `5.1.0`
- Status: active
- Complete operation inventory:
  [TECH-api-specification.md](./TECH-api-specification.md)

## 1. Surface Selection

| Consumer | Surface | SDK | Authentication |
| --- | --- | --- | --- |
| PC, H5, Flutter and product app | App API | `@sdkwork/agents-app-sdk` / `sdkwork_agents_app_sdk` | dual token |
| Operator console and automation | Backend API | `@sdkwork/agents-backend-sdk` | dual token/operator context |
| External integration | Open API | `@sdkwork/agents-sdk` | `X-API-Key` |

Do not call Backend API from user-facing packages. Do not place Open API keys in
the app/backend token manager. Do not deep-import generated transport files.

## 2. Resource Map

| Resource | Purpose | Primary operation namespace |
| --- | --- | --- |
| Agents | Managed definitions and lifecycle | `agents.*` |
| Provider bindings | Provider/model selection | `agents.providerBindings.*` |
| Projects | Reusable orchestration context | `agents.projects.*` |
| Sessions | Durable execution context | `agents.sessions.*` |
| Turns | Idempotent execution lifecycle | `agents.turns.*` |
| Session items | Ordered typed execution facts | `agents.sessionItems.*` |
| Item feedback | Per-user item assessment | `agents.itemFeedback.*` |
| Interactions | Approval and user-question pause points | `agents.interactions.*` |
| Checkpoints | Resume points | `agents.checkpoints.*` |
| Runtime bindings | Session runtime selection | `agents.sessionRuntimeBindings.*` |
| Tasks | Durable one-time/cron definitions, Runs and Attempts | `agents.tasks.*` |
| Composition slots | Independent capability references | `agents.compositionSlots.*` |

Surface-specific availability is authoritative in the generated inventory.

## 3. TypeScript App SDK

```typescript
import {
  completeAgentTurn,
  createClient,
  type CreateAgentSessionRequest,
  type CreateAgentTurnRequest,
} from '@sdkwork/agents-app-sdk';

const client = createClient({
  baseUrl: 'https://agents.example.com/app/v3/api',
  tokenManager,
});

const sessionInput: CreateAgentSessionRequest = {
  sessionKind: 'coding',
  entrySurface: 'pc',
  idempotencyKey: sessionKey,
  payloadHash: sessionPayloadHash,
  requestedAt: new Date().toISOString(),
};

const session = await client.ai.sessions.create(agentId, sessionInput);

const turnInput: CreateAgentTurnRequest = {
  content: prompt,
  turnMode: 'interactive',
  idempotencyKey: turnKey,
  payloadHash: turnPayloadHash,
  requestedAt: new Date().toISOString(),
};

const result = await completeAgentTurn(
  client,
  agentId,
  session.item.sessionId,
  turnInput,
);
```

`result` is `{ session, turn, items }`. List/retrieve operations use
`client.ai.agents.sessions`, `client.ai.agents.turns`,
`client.ai.agents.sessionItems`, `client.ai.agents.itemFeedback` and
`client.ai.agents.interactions`.

Session Item history is an opaque keyset cursor list. Fetch the newest page
with `sort: '-sequence'`, render accepted records back in chronological order,
and continue only with the returned `pageInfo.nextCursor`. Never decode a
cursor or substitute an offset:

```typescript
const page = await client.ai.agents.sessionItems.list(agentId, sessionId, {
  pageSize: 50,
  sort: '-sequence',
});

const earlier = page.pageInfo.hasMore && page.pageInfo.nextCursor
  ? await client.ai.agents.sessionItems.list(agentId, sessionId, {
      cursor: page.pageInfo.nextCursor,
      pageSize: 50,
      sort: '-sequence',
    })
  : null;
```

Streaming:

```typescript
for await (const event of client.ai.agents.turns.stream(
  agentId,
  sessionId,
  turnInput,
  { stream: true },
)) {
  if (event.eventType === 'delta') renderDelta(event.delta ?? '');
  if (event.eventType === 'completion') commit(event.response);
}
```

## 4. Flutter App SDK

Dependency package: `sdkwork_agents_app_sdk`.

```dart
import 'package:sdkwork_agents_app_sdk/sdkwork_agents_app_sdk.dart';

final client = SdkworkAppClient.withBaseUrl(
  baseUrl: 'https://agents.example.com',
  authToken: authToken,
  accessToken: accessToken,
);

final session = await client.ai.agentsSessionsCreate(
  agentId,
  CreateAgentSessionRequest(
    sessionKind: 'coding',
    entrySurface: 'flutter',
    idempotencyKey: sessionKey,
    payloadHash: sessionPayloadHash,
    requestedAt: DateTime.now().toUtc().toIso8601String(),
  ),
);

final events = client.ai.agentsTurnsStream(
  agentId,
  sessionId,
  CreateAgentTurnRequest(
    content: prompt,
    turnMode: 'interactive',
    idempotencyKey: turnKey,
    payloadHash: turnPayloadHash,
    requestedAt: DateTime.now().toUtc().toIso8601String(),
  ),
  true,
);
```

The Agents Flutter mobile core accepts the canonical full App surface URL and
normalizes it before constructing this generated client. Feature code receives
the client through bootstrap injection.

## 5. Project Discovery And Import

Project lists are Workspace-scoped offset pages. Use
`client.ai.agents.projects.list({ workspaceId, q })` for bounded server search
and `nameExact` for case-insensitive exact-name resolution. Retrieve a known
Session directly with
`client.ai.agents.projectSessions.retrieve(projectId, sessionId)` instead of
scanning every Project Session page.

Normal Project and Session activity refresh reads the cursor snapshot.
The Workspace Session Inbox additionally issues bounded, deduplicated
`client.ai.agents.projectSessions.synchronize(projectId)` passes so
provider-owned Sessions converge without blocking the activity read, and the
backend serves repeat synchronizations from a 60-second process-local refresh
cache aligned with the client-side deduplication TTL, so the background loop
never re-scans the provider session store in steady state. A manual project
refresh treats the import as best-effort and continues with the persisted
inventory when the import fails or exceeds its budget. An explicit folder
import or re-import requires the synchronization outcome. The command derives
the working directory from the trusted server-side runtime binding and
returns `synchronizedSessionCount`, `skippedSessionCount`,
`failedSessionCount`, and bounded aggregate `issues`. A nonzero skipped or
failed count is a completed partial result that the client must surface; it
is not transport success for every provider record.

## 6. Contract Rules

- Session creation requires `sessionKind`, `entrySurface`, `idempotencyKey`,
  `payloadHash` and `requestedAt`.
- Turn creation requires `content`, `turnMode`, `idempotencyKey`, `payloadHash`
  and `requestedAt`.
- Retry the same command with the same key and hash.
- Treat a key/hash conflict as a client correctness error.
- Use `expectedVersion` for optimistic commands.
- Offset lists use `page` and language-level `pageSize`; cursor lists use only
  the returned opaque `cursor` and `pageSize`. The SDK serializes `page_size`.
- Validate `PageInfo.mode` and mode-specific continuation fields before
  committing list state. Reject a non-progressing cursor.
- Treat Session Items as execution facts, not IM communication resources.
- Persist only stable Agents identifiers in a consuming product.

## 7. Authorities And Verification

- Authored OpenAPI:
  `crates/sdkwork-intelligence-agents-service/specs/openapi/`
- SDK family manifests: `sdks/sdkwork-agents-*-sdk/sdk-manifest.json`
- Database:
  `crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`

```powershell
node scripts/generate-agents-api-docs.mjs --check
node scripts/check-agent-sdk-workspace.mjs
node sdks/workspace-agent-sdkgen.mjs --mode dry-run
```
