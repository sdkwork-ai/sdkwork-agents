//! The single Playground composition surface for SDKWork Agents PC.
//!
//! `AgentsPlayground` is the ONLY playground shell: it composes the
//! `AgentsWorkbench` system (chat / inspiration / creative generation /
//! assets / canvas / agents sidebar) with playground-level defaults. Host
//! applications (e.g. the Cloud Router portal) embed this package and inject
//! their runtime bindings — SDK clients, balance, token plan, login
//! redirect — through `configureAgentsPlaygroundRuntime`. There must be no
//! second playground implementation: host-specific adapters stay in the
//! host, and every playground capability lands here.

import { AgentsWorkbench, type AgentsWorkbenchProps } from '@sdkwork/agents-pc/workbench';

export interface AgentsPlaygroundProps extends AgentsWorkbenchProps {
  /**
   * Tabs hidden by the embedding host. Defaults to hiding `presentation`
   * (a local demo surface with no backend integration), mirroring the
   * canonical Cloud Router playground composition.
   */
  hiddenTabs?: AgentsWorkbenchProps['hiddenTabs'];
}

/**
 * The canonical Playground composition.
 *
 * Rendering contract: the host owns the surrounding page chrome (navbar,
 * overlay inset); this component owns the full playground workbench below
 * it. `overlayTopInset` pushes the workbench below the host navbar.
 */
export function AgentsPlayground({
  hiddenTabs = ['presentation'],
  overlayTopInset = '0px',
  showSidebarLogo = false,
  viewportMode,
}: AgentsPlaygroundProps) {
  return (
    <AgentsWorkbench
      hiddenTabs={hiddenTabs}
      overlayTopInset={overlayTopInset}
      showSidebarLogo={showSidebarLogo}
      viewportMode={viewportMode}
    />
  );
}
