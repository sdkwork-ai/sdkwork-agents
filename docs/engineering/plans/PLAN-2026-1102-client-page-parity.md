# Client Page Parity Plan

- Version: `1.0.0`
- Status: active
- Owner: `agents-platform`
- Updated: `2026-11-02`
- Specs: [APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md](../../../../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md), [APP_H5_ARCHITECTURE_SPEC.md](../../../../sdkwork-specs/APP_H5_ARCHITECTURE_SPEC.md), [MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md](../../../../sdkwork-specs/MINI_PROGRAM_APP_ARCHITECTURE_SPEC.md), [FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md](../../../../sdkwork-specs/FLUTTER_APP_MOBILE_ARCHITECTURE_SPEC.md), [HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md](../../../../sdkwork-specs/HARMONY_APP_MOBILE_ARCHITECTURE_SPEC.md)
- Contract: [client-page-parity.spec.json](../../../specs/client-page-parity.spec.json)
- Verification: `node --test tests/contract/client-page-parity.contract.test.mjs`

## 1. Objective

Every SDKWork Agents client root must implement the PC workbench page system.
The PC workbench is the reference surface; H5, the WeChat mini program, Flutter
(Android and iOS targets), and HarmonyOS native align by **route identity**,
not by identical physical paths
(`APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md` section 7).

A page is "commercially aligned" only when all of the following hold:

1. its canonical `app.agents.*` page id is declared in the client's route
   registry;
2. a real screen resolves that id and reads data through the client's SDK or
   service-port boundary (no fabricated data, no raw HTTP);
3. loading, empty, error, and retry states are implemented and localized in
   `zh-CN` and `en-US`;
4. the client's typecheck/analyze/tests/build pass.

## 2. Reference Page Inventory (PC)

| Page id | PC entry | PC surface |
| --- | --- | --- |
| `app.agents.conversation.chat` | `chat_session` tab | Chat workbench |
| `app.agents.conversation.list` | chat sidebar | Session history |
| `app.agents.catalog.list` | `agents` tab | Agent catalog and workspace |
| `app.agents.catalog.editor` | `agents` tab | Agent create/edit |
| `app.agents.inspiration.discover` | `inspiration` tab | Inspiration feed |
| `app.agents.creative.studio` | `creative` tab | Generation studio |
| `app.agents.assets.list` | `assets` tab | Asset library |
| `app.agents.assets.detail` | assets modal | Asset detail and slideshow |
| `app.agents.canvas.board` | `canvas` tab | Infinite canvas |
| `app.agents.membership.catalog` | "开会员" + token-plan overlay | Membership catalog and checkout |
| `app.agents.profile.index` | avatar | User profile (profile, security, billing, usage, developer tabs) |
| `app.agents.settings.index` | settings modal | Settings |
| `app.agents.projects.list` | chat sidebar | Project list |
| `app.agents.library.list` | chat file library | File library |
| `app.agents.automation.index` | mobile only | Automation schedules |

`app.agents.automation.index` has no PC entry; it stays registered so mobile
roots do not lose an existing capability.

## 3. Mobile Navigation Model

Five bottom tabs mirror the PC sidebar's primary surfaces:

1. 对话 — `app.agents.conversation.chat`
2. 灵感 — `app.agents.inspiration.discover`
3. 生成 — `app.agents.creative.studio`
4. 资产 — `app.agents.assets.list`
5. Agent — `app.agents.catalog.list`

The avatar entry on every tab opens `app.agents.profile.index`, which links to
membership, projects, library, automation, history, and settings. This keeps
every PC page reachable without a seven-slot tab bar.

## 4. Current State Audit (2026-11-02)

The table tracks route identity plus a real screen, not only a registered id.
`M2` means every contract page has a mounted screen on that root.

| Client | Shell | Pages present | Milestone |
| --- | --- | --- | --- |
| PC | 6 sidebar tabs + membership overlay | all 15 reference pages | M2 |
| H5 | 5 tabs + tab-bar profile entry; stack routes for the profile hub | all 15: chat, history, catalog, editor, inspiration, creative, assets list/detail, canvas, membership, profile, settings, projects, library, automation | M2 |
| Mini program | 5 tabs + avatar entry; subpackage for history | all 15: chat, history, catalog, editor, inspiration, creative, assets list/detail, canvas, membership, profile, settings, projects, library, automation | M2 |
| Flutter | 5 tabs + profile hub; Android and iOS platform shells | all 15: chat, history, catalog, editor, inspiration, creative, assets list/detail, canvas, membership, profile, settings, projects, library, automation | M2 |
| HarmonyOS | 5 tabs + profile hub; router pages | all 15 route ids delivered; M2 pages are mounted router destinations with explicit not-delivered transports | M2 |

Capability gap: the H5 and mini program roots compose real `/ai/assets` and
`/ai/tools` transports (asset library and creative studio). The inspiration
feed, membership catalog, and checkout remain honest `unavailable` states on
every mobile root because no mobile root composes the feeds, membership, or
order transport yet (M3). No screen fabricates a plan, price, feed card, or
asset.

Commercial gaps still open:

1. Inspiration, membership, and checkout need real mobile transports (M3); the
   membership screen routes users to the PC surface instead of inventing one.
   HarmonyOS has no adapted ArkTS SDK transport for any capability, so its M2
   pages render explicit not-delivered states.
2. Visual/interaction evidence against PC reference screenshots (M4) and
   per-store release metadata/signing (M5) are not yet recorded.
3. Flutter canvas and HarmonyOS canvas are read-only projections; mobile editing
   remains a PC/tablet capability by design.

## 5. Milestones

| Milestone | Scope | Exit criteria |
| --- | --- | --- |
| M1 | PC-primary pages on all four mobile roots; shared tab model; profile hub | Parity contract M1 assertions pass for every client; each client typecheck/analyze passes |
| M2 | Secondary pages (history, editor, asset detail, canvas, projects, library, settings, automation) on all roots | Parity contract enforces M2 ids |
| M3 | Real SDK transports per capability (assets, generations, membership) on each root; account sub-pages (security, billing, usage, developer) inside profile | No port left in "unavailable" state for a capability the backend exposes; account self-service matches PC profile tabs (H5/mini program membership catalog + inspiration now real; checkout and Flutter/Harmony transports remain) |
| M4 | Visual and interaction parity pass against PC reference screenshots | Per-platform screenshot evidence in `docs/engineering/reviews/` |
| M5 | Commercial release gates: store metadata, signing references, privacy/permission declarations, offline/error/rate-limit behavior | `pnpm check` plus per-platform release verification |

## 6. Platform Decisions

- **iOS and Android native**: the repository declares `APP_ANDROID`/`APP_IOS`
  only through `apps/sdkwork-agents-flutter-mobile`
  (`agent.app.config.json`). No `-android-mobile` or `-ios-mobile` root exists.
  M1 treats Flutter's Android/iOS targets as those native surfaces and adds the
  missing platform shells. Introducing separate native roots requires a
  governance decision and new root standards.
- **Canvas on phones**: the PC canvas is a pointer-and-keyboard editor. Mobile
  roots ship a read-only/viewport-constrained canvas surface for M2; full
  editing stays a PC/tablet capability until a mobile canvas interaction spec is
  approved.
- **Data honesty**: where a client has no generated SDK transport for a
  capability, the page renders an explicit unavailable state with retry and
  records the gap in M3. It must never show fabricated content.

## 7. Verification

```bash
node --test tests/contract/client-page-parity.contract.test.mjs
pnpm --filter @sdkwork/agents-pc typecheck
pnpm --filter @sdkwork/agents-h5 typecheck
pnpm --filter @sdkwork/agents-mini-program typecheck
cd apps/sdkwork-agents-flutter-mobile && flutter analyze
node --test apps/sdkwork-agents-harmony-mobile/tests/harmony-surface-contract.test.mjs
```

## 9. M1/M2 Evidence (2026-11-02)

| Client | Evidence | Result |
| --- | --- | --- |
| Contract | `node --test tests/contract/client-page-parity.contract.test.mjs` | 6/6 pass; all five clients now declare `deliveredThrough: M2` and every M2 id |
| PC | `pnpm --filter @sdkwork/agents-pc typecheck`; `tsx --test tests/workbench-modules.test.ts` | pass; 14/14 |
| H5 | `pnpm --filter @sdkwork/agents-h5 typecheck` / `build` / `test`; `check-frontend-composition` | pass; build `dist/standalone/prod` |
| Mini program | `typecheck`; `build:mini-program`; `scripts/mini-program-runtime-contract.test.mjs` | pass; runtime 336.6 kB, 15 pages |
| Flutter | `flutter analyze`; `flutter test`; `flutter build apk --debug`; `dart format --set-exit-if-changed` | analyze clean; 16/16; APK built |
| HarmonyOS | `tests/harmony-surface-contract.test.mjs`; `check-frontend-composition` | pass (static only) |

Transport status after M3 progress:

- Real: PC all surfaces; H5 conversation/catalog/library/projects/automation/assets/creative/profile/inspiration/membership catalog + cashier checkout; mini program assets/creative/profile/inspiration/membership catalog; Flutter conversation/catalog/library/projects/automation/assets/profile/editor/assets-detail/canvas; Harmony profile/settings plus mounted M2 pages.
- Explicit `unavailable`: mini program membership checkout (no MP cashier/payment product composed); Flutter inspiration/creative/membership; Harmony conversation/catalog/inspiration/creative/assets/membership (no ArkTS SDK target).

M2 evidence delivered in this pass:

- H5: `/assets/:assetId` route, read-only `/canvas` projection of the PC
  browser-local workflow, `/settings` (environment, cache clear, sign-out),
  `/conversation/history` with session open/delete, and the catalog editor route
  identity. The profile hub exposes membership, projects, library, automation,
  history, and settings. Membership now reads the real catalog and creates a
  `mobile_cashier_h5` order through `@sdkwork/order-service`, redirecting to the
  returned cashier URL.
- Mini program: first-class `pages/assets/detail` (deep link + library tap) and
  read-only `pages/canvas/index` reachable from the creative studio; both are
  projected from shell route contributions and asserted by
  `scripts/mini-program-runtime-contract.test.mjs`.
- Flutter: first-class `/assets/detail` (tile tap passes the resolved asset),
  `/agents/editor` (name/description update through `AgentCatalogService`), and
  `/canvas` (read-only preference-backed projection). The navigation contract
  test now expects all ten non-tab stack routes.
- HarmonyOS: all seven M2 route ids are `delivered` with page files, route-map
  entries, and main-pages registration; the pages are mounted destinations that
  render explicit not-delivered transport states because this root has no ArkTS
  SDK target.

Remaining commercial work is M3/M4/M5: real inspiration/membership/checkout
transports, account sub-pages, screenshot parity evidence, and store release
gates. H5 bundle size (1.47 MB / 410 kB gzip) remains a tracked M4 performance
item.

Flutter Android smoke evidence (emulator, debug): 5-tab bar, profile hub with six links, editor, asset detail, read-only canvas, localized unavailable states, persisted settings language. `flutter build apk --debug` passes after the M2 additions. iOS build and HarmonyOS build remain toolchain-blocked.

## 10. Risks

- Mobile capability packages must not import PC UI packages
  (`APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md` section 1); screens are
  re-authored per stack.
- Mini program page registration is physical (`app.json` + generated runtime),
  so a route id is only real once `pages/` and the runtime bundle carry it.
- HarmonyOS toolchain (`hvigor`/`ohpm`/DevEco signing) is absent from this
  environment; Harmony evidence is static-contract plus ArkTS source review.
- Adding SDK packages to a root changes its `component.spec.json` inventory and
  must run `check:frontend-composition` and `check:component-port-bindings`.

## 11. Theme Direction: `tech-blue-flat`

Authority: [client-theme.spec.json](../../../specs/client-theme.spec.json),
gate: `node --test tests/contract/client-theme.contract.test.mjs`.

The product ships one visual theme across every client root:

| Token | Value | Role |
| --- | --- | --- |
| `primary` | `#165DFF` | Primary actions, active tabs, links |
| `primaryHover` / `primaryPressed` | `#4080FF` / `#0E42D2` | Interaction states |
| `primarySubtle` | `#E8F0FF` | Selected pills, badges |
| `background` | `#F4F7FC` | Page canvas |
| `surface` | `#FFFFFF` | Cards, rows, tab bars, sheets |
| `surfaceMuted` / `surfaceStrong` | `#EAF1FB` / `#DCE8F9` | Inputs, chips, media placeholders |
| `rail` | `#0A1B3D` | Desktop navigation rail |
| `textPrimary` / `textSecondary` / `textTertiary` | `#0A1B3D` / `#5B6B88` / `#8A97AD` | Text ramp |
| `border` | `transparent` | No decorative 1px lines |
| `elevation.shadow` | `none` | Flat; depth is tonal |
| `focus` | 2px `#165DFF`, offset 2 | Accessibility outline, not a border |

Per-client token layers: PC CSS custom properties (`--ag-*`), H5 CSS variables,
mini program WXSS custom properties, Flutter
`packages/sdkwork_agents_flutter_mobile_commons/lib/src/theme/tech_blue_theme.dart`,
HarmonyOS `packages/sdkwork-agents-harmony-mobile-commons/src/main/ets/theme/DesignTokens.ets`.

Rules enforced by review and the theme gate:

- No visible decorative border on shell, cards, inputs, tab bars, modals, or rows.
- No box-shadow/elevation on surfaces; dialogs use the navy scrim only.
- No gradients in shell or primary surfaces.
- Primary actions are `#165DFF` with white foreground; focus rings stay visible.
- Selection is never conveyed by colour alone (icon/weight also changes).
