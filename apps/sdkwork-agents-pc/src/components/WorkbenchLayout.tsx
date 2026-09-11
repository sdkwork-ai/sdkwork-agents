import { lazy, Suspense, useCallback, useEffect, useState, type ComponentType, type CSSProperties, type LazyExoticComponent } from 'react';
import { useTranslation } from 'react-i18next';
import { X } from 'lucide-react';
import {
  AGENTS_OPEN_TOKEN_PLAN_EVENT,
  AGENTS_TOKEN_PLAN_CLOSED_EVENT,
} from '@sdkwork/agents-pc-chat';
import { GlobalSidebar } from './GlobalSidebar';
import { AgentStatusIndicator } from './AgentStatusIndicator';
import { DEFAULT_WORKBENCH_TAB, isWorkbenchTab, type WorkbenchTab } from './workbenchTabs';

const AgentWorkspace = lazy(async () => {
  const [agentsModule, knowledgebaseRuntime] = await Promise.all([
    import('../agents'),
    import('../bootstrap/knowledgebaseRuntime'),
  ]);
  knowledgebaseRuntime.initializeAgentsKnowledgebaseRuntime();
  return { default: agentsModule.AgentWorkspace };
});
const ChatView = lazy(() => import('@sdkwork/agents-pc-chat/ChatView').then((module) => ({ default: module.ChatView })));
const InspirationView = lazy(() => import('@sdkwork/agents-pc-inspiration').then((module) => ({ default: module.InspirationView })));
const CreativeView = lazy(() => import('@sdkwork/agents-pc-creative').then((module) => ({ default: module.CreativeView })));
const AssetsView = lazy(() => import('@sdkwork/agents-pc-assets').then((module) => ({ default: module.AssetsView })));
const CanvasView = lazy(() => import('@sdkwork/agents-pc-canvas').then((module) => ({ default: module.CanvasView })));
const PresentationView = lazy(() => import('@sdkwork/agents-pc-presentation').then((module) => ({ default: module.PresentationView })));
const AgentsTokenPlanView = lazy(() => import('@sdkwork/agents-pc-membership').then((module) => ({ default: module.AgentsTokenPlanView })));

const WORKBENCH_VIEW_BY_TAB: Record<WorkbenchTab, LazyExoticComponent<ComponentType>> = {
  agents: AgentWorkspace,
  chat_session: ChatView,
  inspiration: InspirationView,
  creative: CreativeView,
  assets: AssetsView,
  canvas: CanvasView,
  presentation: PresentationView,
};

const AVATAR_COLOR_TEMPLATES = [
  'bg-[#1890ff] text-white',
  'bg-emerald-500 text-white',
  'bg-violet-500 text-white',
  'bg-orange-500 text-white',
  'bg-rose-500 text-white',
  'bg-zinc-800 border border-zinc-700 text-white'
];

export type WorkbenchViewportMode = 'embedded' | 'fixed';

type WorkbenchLayoutStyle = CSSProperties & {
  '--sdkwork-agents-overlay-top-inset': string;
};

interface WorkbenchLayoutProps {
  /** Tabs hidden by the embedding host (e.g. surfaces with no backend yet). */
  hiddenTabs?: readonly WorkbenchTab[];
  overlayTopInset?: string;
  showSidebarLogo?: boolean;
  viewportMode?: WorkbenchViewportMode;
}

/** Loads the membership runtime shared by both Token Plan surfaces. */
function useTokenPlanRuntime(): () => void {
  return useCallback(() => {
    void import('../bootstrap/tokenPlanRuntime').then(({ initializeAgentsTokenPlanRuntime }) => {
      initializeAgentsTokenPlanRuntime();
    });
  }, []);
}

export const WorkbenchLayout = ({
  hiddenTabs = [],
  overlayTopInset = '0px',
  showSidebarLogo = true,
  viewportMode = 'fixed',
}: WorkbenchLayoutProps) => {
  const [activeTab, setActiveTab] = useState<WorkbenchTab>(DEFAULT_WORKBENCH_TAB);
  const [isTokenPlanOpen, setIsTokenPlanOpen] = useState(false);
  const [isTokenPlanOverlayOpen, setIsTokenPlanOverlayOpen] = useState(false);
  const { t: tCommon } = useTranslation('common');
  const initializeTokenPlanRuntime = useTokenPlanRuntime();

  const [username, setUsername] = useState(() => localStorage.getItem('profile_username') || tCommon('mockUserName'));
  const [avatarIndex, setAvatarIndex] = useState(() => parseInt(localStorage.getItem('profile_avatar_index') || '0', 10));

  const closeTokenPlanOverlay = useCallback(() => {
    setIsTokenPlanOverlayOpen(false);
    // A purchase (or an abandoned checkout) can change the balance; let the
    // chat surface re-read it once the overlay is gone.
    window.dispatchEvent(new CustomEvent(AGENTS_TOKEN_PLAN_CLOSED_EVENT));
  }, []);

  useEffect(() => {
    const handleStorageChange = () => {
      setUsername(localStorage.getItem('profile_username') || tCommon('mockUserName'));
      setAvatarIndex(parseInt(localStorage.getItem('profile_avatar_index') || '0', 10));
    };
    const handleSwitchTab = (e: Event) => {
      const customEvent = e as CustomEvent<{ tab?: unknown }>;
      if (isWorkbenchTab(customEvent.detail?.tab)) {
        setIsTokenPlanOpen(false);
        setActiveTab(customEvent.detail.tab);
      }
    };
    const handleOpenTokenPlanOverlay = () => {
      initializeTokenPlanRuntime();
      setIsTokenPlanOverlayOpen(true);
    };
    window.addEventListener('storage', handleStorageChange);
    window.addEventListener('switch-tab', handleSwitchTab);
    window.addEventListener(AGENTS_OPEN_TOKEN_PLAN_EVENT, handleOpenTokenPlanOverlay);
    const interval = setInterval(handleStorageChange, 800);
    return () => {
      window.removeEventListener('storage', handleStorageChange);
      window.removeEventListener('switch-tab', handleSwitchTab);
      window.removeEventListener(AGENTS_OPEN_TOKEN_PLAN_EVENT, handleOpenTokenPlanOverlay);
      clearInterval(interval);
    };
  }, [tCommon, initializeTokenPlanRuntime]);

  useEffect(() => {
    if (!isTokenPlanOverlayOpen) {
      return;
    }
    // Lock background scrolling while the full-screen purchase page is open.
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        closeTokenPlanOverlay();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      document.body.style.overflow = previousOverflow;
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [isTokenPlanOverlayOpen, closeTokenPlanOverlay]);

  const avatarBg = AVATAR_COLOR_TEMPLATES[avatarIndex] || AVATAR_COLOR_TEMPLATES[0];
  const safeActiveTab = hiddenTabs.includes(activeTab) ? DEFAULT_WORKBENCH_TAB : activeTab;
  const ActiveWorkspace = WORKBENCH_VIEW_BY_TAB[safeActiveTab];
  const workbenchLayoutStyle: WorkbenchLayoutStyle = {
    '--sdkwork-agents-overlay-top-inset': overlayTopInset,
  };

  return (
    <div
      className={`sdkwork-agents-workbench flex min-h-0 w-full bg-[#f5f5f5] font-sans text-gray-900 dark:bg-[#191919] dark:text-gray-100 overflow-hidden ${viewportMode === 'fixed' ? 'h-[100dvh]' : 'h-full'}`}
      style={workbenchLayoutStyle}
    >
      <AgentStatusIndicator />
      <GlobalSidebar 
        activeTab={safeActiveTab} 
        avatarBg={avatarBg} 
        hiddenTabs={hiddenTabs}
        isTokenPlanOpen={isTokenPlanOpen}
        onOpenTokenPlan={() => {
          initializeTokenPlanRuntime();
          setIsTokenPlanOpen(true);
        }}
        setActiveTab={(tab) => {
          setIsTokenPlanOpen(false);
          setActiveTab(tab);
        }}
        showSidebarLogo={showSidebarLogo}
        username={username} 
      />

      {/* Main Content Area */}
      <div className="flex-1 w-0 flex flex-col relative overflow-hidden bg-[#f5f5f5] dark:bg-[#191919]">
        <Suspense fallback={<div className="flex flex-1 items-center justify-center text-sm text-gray-500">正在加载工作台…</div>}>
          {isTokenPlanOpen ? <AgentsTokenPlanView /> : <ActiveWorkspace />}
        </Suspense>
      </div>

      {isTokenPlanOverlayOpen && (
        <div
          className="sdkwork-agents-token-plan-overlay fixed inset-0 z-[9999] flex h-[100dvh] w-full flex-col bg-black/70 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          aria-label={tCommon('tokenPlan', 'Token Plan')}
        >
          <div className="relative flex h-full w-full flex-col overflow-hidden bg-[#0e0e11] shadow-2xl">
            <button
              aria-label={tCommon('close', 'Close')}
              className="absolute right-4 top-4 z-10 flex h-9 w-9 items-center justify-center rounded-full border border-white/10 bg-white/5 text-zinc-300 transition-colors hover:bg-white/15 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white/60"
              onClick={closeTokenPlanOverlay}
              title={tCommon('close', 'Close')}
              type="button"
            >
              <X aria-hidden size={18} />
            </button>
            <div className="min-h-0 flex-1 overflow-y-auto">
              <Suspense fallback={<div className="flex h-full items-center justify-center text-sm text-zinc-400">正在加载会员方案...</div>}>
                <AgentsTokenPlanView />
              </Suspense>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
