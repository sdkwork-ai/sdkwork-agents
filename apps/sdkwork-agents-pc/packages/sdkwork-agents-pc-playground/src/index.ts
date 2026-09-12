//! `@sdkwork/agents-pc-playground` — the single Playground package.
//!
//! Hosts embed `AgentsPlayground` and inject runtime bindings through
//! `configureAgentsPlaygroundRuntime` (re-exported verbatim from the
//! workbench runtime). Everything playground-related composes here; hosts
//! must not wrap alternative shells (e.g. generation-only playgrounds) —
//! new capabilities land in the workbench system and surface through this
//! package.

export { AgentsPlayground, type AgentsPlaygroundProps } from './AgentsPlayground';
export {
  configureAgentsWorkbenchRuntime as configureAgentsPlaygroundRuntime,
  type AgentsWorkbenchRuntime as AgentsPlaygroundRuntime,
} from '@sdkwork/agents-pc/workbench';
export {
  DEFAULT_WORKBENCH_TAB,
  isWorkbenchTab,
  SIDEBAR_TABS,
  WORKBENCH_TABS,
  type WorkbenchTab,
} from '@sdkwork/agents-pc/workbench';
export type { ChatBalancePort } from '@sdkwork/agents-pc/workbench';
