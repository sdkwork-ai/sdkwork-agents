# REVIEW-20261102: Client page parity and commercial readiness audit

- Status: **No-Go for full commercial completion; Go for the delivered M2/M3 scope.**
- Date: 2026-11-02
- Scope: PC reference workbench and the H5, WeChat mini program, Flutter mobile
  (Android/iOS targets), and HarmonyOS client roots.
- Authority: `specs/client-page-parity.spec.json`,
  `specs/client-theme.spec.json`,
  `APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md`,
  `APP_FLUTTER_UI_SPEC.md`, `APP_MINI_PROGRAM_UI_SPEC.md`,
  `APP_HARMONY_UI_SPEC.md`.

## 1. Page matrix

All 15 canonical page ids exist on every declared mobile root. "Real" means the
page is backed by a capability service and an injected SDK/port; "mounted" means
the route exists and renders an explicit localized unavailable/empty state.

| Page id | H5 | Mini program | Flutter | HarmonyOS |
| --- | --- | --- | --- | --- |
| `app.agents.conversation.chat` | Real | Real | Real | Mounted |
| `app.agents.conversation.list` | Real | Real | Real | Mounted |
| `app.agents.catalog.list` | Real | Real | Real | Mounted |
| `app.agents.catalog.editor` | Real | Real | Real | Mounted |
| `app.agents.inspiration.feed` | Real | Real | Unavailable | Mounted |
| `app.agents.creative.studio` | Real | Real | Real (`/ai/tools`) | Mounted |
| `app.agents.assets.list` | Real | Real | Real | Mounted |
| `app.agents.assets.detail` | Real | Real | Real | Mounted |
| `app.agents.canvas.board` | Real (PC localStorage projection) | Real (read-only projection) | Real (preference projection) | Mounted |
| `app.agents.membership.catalog` | Real + cashier checkout + points + points ledger | Real + cashier web-view + points | Unavailable | Mounted |
| `app.agents.profile.index` | Real + usage | Real + usage | Real + usage | Real |
| `app.agents.settings.index` | Real | Real | Real | Real |
| `app.agents.projects.list` | Real | Real | Real | Mounted |
| `app.agents.library.list` | Real | Real | Real | Mounted |
| `app.agents.automation.index` | Real | Real | Real | Mounted |

## 2. Transport matrix

| Capability | H5 | Mini program | Flutter | HarmonyOS |
| --- | --- | --- | --- | --- |
| Agents app SDK (`/ai/**`) | Yes | Yes | Yes | No ArkTS target |
| Drive app SDK | Yes | Yes | Yes | No ArkTS target |
| Membership app SDK | Yes | Yes | No Flutter target | No ArkTS target |
| Feeds open SDK | Yes | Yes | No Flutter target | No ArkTS target |
| Order app service | Yes | Yes | No Flutter target | No ArkTS target |
| Usage summary | Yes | Yes | Yes | No (no transport) |

No client root uses raw HTTP or hand-assembled auth headers for these
capabilities. Where a language target does not exist, the page reports an
explicit unavailable state instead of inventing data.

## 3. Verification evidence

Commands run from the repository root on 2026-11-02:

| Gate | Result |
| --- | --- |
| H5 typecheck / build | pass (`dist/standalone/prod`) |
| H5 test | 9/9 pass |
| H5 `client-theme` contract (light/dark/system) | 8/8 pass |
| Mini program typecheck | pass |
| Mini program runtime build | pass (508.6 kB) |
| Mini program runtime contract | pass |
| Flutter analyze | 0 issues |
| Flutter test | 37/37 pass |
| Flutter APK debug build | pass |
| HarmonyOS surface contract | pass (static) |
| `client-page-parity` contract | 6/6 pass |
| `client-surface-readiness` contract | pass |
| `check-frontend-composition` / `check-component-port-bindings` | pass |

## 4. Release gates already encoded

- Flutter Android release signing reads `android/key.properties`; the example
  file documents the expected keys. Missing signing material falls back to the
  debug key for local smoke tests only.
- Flutter Android main manifest declares `INTERNET` and disables app-data
  backup.
- Flutter iOS ships `PrivacyInfo.xcprivacy` with the UserDefaults required-reason
  API and declares `ITSAppUsesNonExemptEncryption=false`.
- H5 and mini program cloud production environments must use HTTPS and must not
  point at local hosts.

## 5. Remaining blockers

1. **Flutter inspiration and membership.** `sdkwork-membership-app-sdk` and
   `sdkwork-feeds-sdk` currently generate TypeScript only. Adding Flutter
   targets is a generated-SDK ownership change and needs owner review before
   regeneration; client-side raw HTTP is not an acceptable substitute.
2. **HarmonyOS capability transports.** No ArkTS SDK target exists for the
   Agents app SDK, so every non-settings capability remains a mounted
   unavailable state. Harmony evidence is static only: this environment has no
   hvigor/ohpm/DevEco signing.
3. **M4 visual evidence.** Per-page PC-versus-mobile reference screenshots have
   not been produced. The theme token contract passes, but it is not a visual
   diff.
4. **M5 store operations.** Real App Store/WeChat/Android signing identities,
   WeChat business-domain registration for the cashier web-view, and the
   privacy-data-type legal declaration remain release-environment tasks.
5. **Account sub-pages.** Usage is now shown on H5, mini program, and Flutter
   profiles; security sessions, invoices, and full order history are not yet
   implemented.

## 6. Decision

- **Go:** M2 page coverage, real H5/mini program/Flutter transports for the
  composed capabilities, theme token alignment, and encoded release gates.
- **No-Go:** declaring the mobile surfaces fully commercial-ready while
  Flutter inspiration/membership and HarmonyOS transports are unavailable and
  M4/M5 evidence is incomplete.

The next review should start after the SDK owners approve and generate the
missing Flutter/ArkTS targets, or after a product decision to ship those pages as
explicitly unavailable on the affected roots.
