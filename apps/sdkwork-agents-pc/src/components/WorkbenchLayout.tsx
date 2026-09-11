import { lazy, Suspense, useEffect, useState, type ComponentType, type CSSProperties, type LazyExoticComponent } from 'react';
import { useTranslation } from 'react-i18next';
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

export const WorkbenchLayout = ({
  hiddenTabs = [],
  overlayTopInset = '0px',
  showSidebarLogo = true,
  viewportMode = 'fixed',
}: WorkbenchLayoutProps) => {
  const [activeTab, setActiveTab] = useState<WorkbenchTab>(DEFAULT_WORKBENCH_TAB);
  const [isTokenPlanOpen, setIsTokenPlanOpen] = useState(false);
  const { t: tCommon } = useTranslation('common');

  const [username, setUsername] = useState(() => localStorage.getItem('profile_username') || tCommon('mockUserName'));
  const [avatarIndex, setAvatarIndex] = useState(() => parseInt(localStorage.getItem('profile_avatar_index') || '0', 10));

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
    window.addEventListener('storage', handleStorageChange);
    window.addEventListener('switch-tab', handleSwitchTab);
    const interval = setInterval(handleStorageChange, 800);
    return () => {
      window.removeEventListener('storage', handleStorageChange);
      window.removeEventListener('switch-tab', handleSwitchTab);
      clearInterval(interval);
    };
  }, [tCommon]);

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
          void import('../bootstrap/tokenPlanRuntime').then(({ initializeAgentsTokenPlanRuntime }) => {
            initializeAgentsTokenPlanRuntime();
            setIsTokenPlanOpen(true);
          });
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
    </div>
  );
};
