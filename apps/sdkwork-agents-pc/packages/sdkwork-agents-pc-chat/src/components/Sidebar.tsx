import React, { useState, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import {
  Plus,
  MessageSquare,
  Trash2,
  Search,
  MoreHorizontal,
  Pin,
  Gift,
  Sparkles,
  X,
  Star,
  Users,
  Bot,
  LayoutGrid,
  Settings,
  Compass,
  Share,
  UserPlus,
  Edit3,
  FolderInput,
  Archive,
  ChevronRight,
  SquarePen,
  Library,
  Folder,
  PanelLeftClose,
  ChevronDown,
  FolderPlus,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { ChatSession, type ChatProject } from "@sdkwork/agents-pc-chat";
import { cn } from "@sdkwork/agents-pc-commons";
import { BirdCoderModal } from "./BirdCoderModal";
import { SearchModal } from "./SearchModal";
import { SidebarSessionItem } from "./SidebarSessionItem";
import { SidebarProjectsSection } from "./SidebarProjectsSection";
import { SidebarProjectContextMenu } from "./SidebarProjectContextMenu";
import { SidebarSessionContextMenu } from "./SidebarSessionContextMenu";

export interface SidebarProps {
  isOpen: boolean;
  toggleSidebar: () => void;
  sessions: ChatSession[];
  currentSessionId: string;
  onNewChat: () => void;
  onSelectSession: (id: string) => void;
  onDeleteSession: (e: React.MouseEvent, id: string) => void;
  onRenameSession: (id: string, newTitle: string) => void;
  onTogglePin: (id: string, pinned: boolean) => Promise<boolean>;
  onMoveSessionToProject: (sessionId: string, project: ChatProject) => void;
  onOpenUserProfile: (tab: "profile" | "billing") => void;
  onOpenFileLibrary: () => void;
  projects: ChatProject[];
  activeProject?: string | null;
  onProjectSelect?: (project: ChatProject) => void;
  onProjectSettings?: (project: ChatProject) => void;
  onProjectCreate?: (title: string) => void;
  onProjectRename?: (project: ChatProject, newTitle: string) => void;
  onProjectDelete?: (project: ChatProject) => void;
}

const AVATAR_COLOR_TEMPLATES = [
  "bg-[#1890ff] text-white",
  "bg-emerald-500 text-white",
  "bg-violet-500 text-white",
  "bg-orange-500 text-white",
  "bg-rose-500 text-white",
  "bg-zinc-800 border border-zinc-700 text-white",
];

export const Sidebar: React.FC<SidebarProps> = ({
  isOpen,
  toggleSidebar,
  sessions,
  currentSessionId,
  onNewChat,
  onSelectSession,
  onDeleteSession,
  onRenameSession,
  onTogglePin,
  onMoveSessionToProject,
  onOpenUserProfile,
  onOpenFileLibrary,
  projects,
  activeProject,
  onProjectSelect,
  onProjectSettings,
  onProjectCreate,
  onProjectRename,
  onProjectDelete,
}) => {
  const { t } = useTranslation("chat");
  const { t: tCommon } = useTranslation("common");

  const [username, setUsername] = useState(
    () => localStorage.getItem("profile_username") || tCommon("mockUserName"),
  );
  const [avatarIndex, setAvatarIndex] = useState(() =>
    parseInt(localStorage.getItem("profile_avatar_index") || "0", 10),
  );
  const [isBirdCoderModalOpen, setIsBirdCoderModalOpen] = useState(false);
  const [isSearchModalOpen, setIsSearchModalOpen] = useState(false);

  // Real-time search filter
  const [searchQuery, setSearchQuery] = useState("");

  // Action feedback/alert state local toast
  const [sidebarToast, setSidebarToast] = useState("");

  const [activeDropdown, setActiveDropdown] = useState<string | null>(null);
  const [dropdownPos, setDropdownPos] = useState({ top: 0, left: 0 });

  useEffect(() => {
    const handleClickOutside = () => {
      setActiveDropdown(null);
    };
    document.addEventListener("click", handleClickOutside);
    // Also close dropdown on scroll
    const handleScroll = () => {
      setActiveDropdown(null);
    };
    document.addEventListener("scroll", handleScroll, true); // true for capture phase to catch all scrolls

    return () => {
      document.removeEventListener("click", handleClickOutside);
      document.removeEventListener("scroll", handleScroll, true);
    };
  }, []);

  const handleDropdownClick = (e: React.MouseEvent, sessionId: string) => {
    e.stopPropagation();
    if (activeDropdown === sessionId) {
      setActiveDropdown(null);
    } else {
      const rect = e.currentTarget.getBoundingClientRect();
      // Position the dropdown 180px (width of menu) to the left of the button's right edge
      setDropdownPos({ top: rect.bottom, left: rect.right - 180 });
      setActiveDropdown(sessionId);
    }
  };

  useEffect(() => {
    const handleStorageChange = () => {
      setUsername(
        localStorage.getItem("profile_username") || tCommon("mockUserName"),
      );
      setAvatarIndex(
        parseInt(localStorage.getItem("profile_avatar_index") || "0", 10),
      );
    };
    window.addEventListener("storage", handleStorageChange);
    const interval = setInterval(handleStorageChange, 800);
    return () => {
      window.removeEventListener("storage", handleStorageChange);
      clearInterval(interval);
    };
  }, [tCommon]);

  const togglePin = (e: React.MouseEvent, sessionId: string) => {
    e.stopPropagation();
    const wasPinned = sessions.find((session) => session.id === sessionId)?.pinned ?? false;
    void onTogglePin(sessionId, !wasPinned)
      .then((updated) => {
        if (updated) {
          setSidebarToast(wasPinned ? t("unpinnedToast") : t("pinnedToast"));
        }
      })
      .catch(() => undefined);
  };

  useEffect(() => {
    if (sidebarToast) {
      const timer = setTimeout(() => setSidebarToast(""), 1500);
      return () => clearTimeout(timer);
    }
  }, [sidebarToast]);

  const avatarBg =
    AVATAR_COLOR_TEMPLATES[avatarIndex] || AVATAR_COLOR_TEMPLATES[0];

  // Filtering based on search query
  const filteredSessions = sessions.filter((session) => {
    const query = searchQuery.toLowerCase();
    return session.title.toLowerCase().includes(query) ||
           session.messages.some(m => m.text.toLowerCase().includes(query));
  });

  // Split sessions into Pinned and Recent categories
  const pinnedSessions = filteredSessions.filter((s) =>
    s.pinned,
  );
  const recentSessions = filteredSessions.filter(
    (s) => !s.pinned,
  );

  return (
    <>
      <div
        className={cn(
          "flex h-full transition-all duration-300 ease-in-out shrink-0 overflow-hidden relative border-r border-transparent dark:border-white/5",
          isOpen ? "w-[280px]" : "w-0",
        )}
      >
        {/* Toast Overlay for Mini actions status */}
        {sidebarToast && (
          <div className="absolute top-16 left-20 right-4 bg-[#2A2A2D] text-xs text-white px-3 py-2 rounded-xl border border-white/5 shadow-xl flex items-center gap-1.5 justify-center z-50 animate-fade-in-down font-medium tracking-wide">
            <Sparkles size={11} className="text-amber-400 animate-pulse" />
            <span>{sidebarToast}</span>
          </div>
        )}

        {/* Chat List Pane */}
        <div className="flex-1 flex flex-col h-full bg-[#1C1C1E] min-w-0 z-10">
          {/* Top Header section */}
          <div className="p-3 pb-2 shrink-0 flex flex-col gap-1">
            <div className="flex items-center justify-between px-2 py-1 mb-2">
              <svg
                width="28"
                height="28"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
                className="text-white"
              >
                <path
                  d="M16.5 12C16.5 14.4853 14.4853 16.5 12 16.5C9.51472 16.5 7.5 14.4853 7.5 12C7.5 9.51472 9.51472 7.5 12 7.5C14.4853 7.5 16.5 9.51472 16.5 12Z"
                  stroke="currentColor"
                  strokeWidth="1.5"
                />
                <path
                  d="M12 2C13.882 2 15.65 2.50244 17.1583 3.37682M20.6232 6.84175C21.4976 8.35002 22 10.118 22 12C22 13.882 21.4976 15.65 20.6232 17.1583M17.1583 20.6232C15.65 21.4976 13.882 22 12 22C10.118 22 8.35002 21.4976 6.84175 20.6232M3.37682 17.1583C2.50244 15.65 2 13.882 2 12C2 10.118 2.50244 8.35002 3.37682 6.84175M6.84175 3.37682C8.35002 2.50244 10.118 2 12 2M12 2V7.5M12 22V16.5M20.6603 7L15.8971 9.75M3.33975 17L8.10289 14.25M20.6603 17L15.8971 14.25M3.33975 7L8.10289 9.75"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                />
              </svg>
              <button
                onClick={toggleSidebar}
                className="p-1.5 text-zinc-400 hover:text-zinc-200 rounded-md transition-colors hover:bg-white/5"
                title={tCommon("closeSidebar")}
              >
                <PanelLeftClose size={18} />
              </button>
            </div>

            <button
              onClick={onNewChat}
              className="flex items-center gap-3 text-zinc-100 font-medium rounded-lg transition-all w-full py-2 px-3 hover:bg-white/5 active:scale-[0.98]"
            >
              <SquarePen size={18} className="text-zinc-200" />
              <span className="text-[14px]">{t("newChat")}</span>
            </button>

            <button 
              onClick={() => setIsSearchModalOpen(true)}
              className="flex items-center gap-3 text-zinc-100 font-medium rounded-lg transition-all w-full py-2 px-3 hover:bg-white/5 active:scale-[0.98]"
            >
              <Search size={18} className="text-zinc-200" />
              <span className="text-[14px]">{t("searchChat")}</span>
            </button>

            <button 
              onClick={onOpenFileLibrary}
              className="flex items-center gap-3 text-zinc-100 font-medium rounded-lg transition-all w-full py-2 px-3 hover:bg-white/5 active:scale-[0.98]"
            >
              <Library size={18} className="text-zinc-200" />
              <span className="text-[14px]">{t("fileLibrary")}</span>
            </button>

            <button
              onClick={() => setIsBirdCoderModalOpen(true)}
              className="flex items-center gap-3 text-zinc-100 font-medium rounded-lg transition-all w-full py-2 px-3 hover:bg-white/5 active:scale-[0.98]"
            >
              <Bot size={18} className="text-zinc-200" />
              <span className="text-[14px]">{t("codex")}</span>
            </button>
          </div>

          {/* Scrollable Conversation List */}
          <div className="flex-1 overflow-y-auto w-full overflow-x-hidden pt-1 pb-4 scrollbar-none space-y-4">
            {/* Modular Projects Section */}
            <SidebarProjectsSection
              projects={projects}
              activeProject={activeProject}
              onProjectSelect={onProjectSelect}
              onProjectSettings={onProjectSettings}
              onProjectCreate={onProjectCreate}
              onProjectRename={onProjectRename}
              onProjectDelete={onProjectDelete}
            />

            <div className="px-4 text-[15px] font-bold text-zinc-100 mt-6 mb-2">
              聊天
            </div>

            {/* Category A: Pinned (已置顶) */}
            {pinnedSessions.length > 0 && (
              <div className="px-2">
                <div className="text-[11px] font-semibold text-zinc-500 px-2.5 mb-1 flex items-center gap-1.5 uppercase tracking-wider">
                  <span>{t("pinned")}</span>
                </div>
                <div className="space-y-0.5">
                  {pinnedSessions
                    .sort((a, b) => b.updatedAt - a.updatedAt)
                    .map((session) => (
                      <SidebarSessionItem
                        key={session.id}
                        session={session}
                        currentSessionId={!activeProject ? currentSessionId : ""}
                        isPinned={true}
                        activeDropdown={activeDropdown}
                        dropdownPos={dropdownPos}
                        onSelectSession={onSelectSession}
                        togglePin={togglePin}
                        onDeleteSession={onDeleteSession}
                        onRenameSession={onRenameSession}
                        handleDropdownClick={handleDropdownClick}
                        setActiveDropdown={setActiveDropdown}
                        t={t}
                        projectsList={projects}
                        onMoveToProject={(project) => {
                          void onMoveSessionToProject(session.id, project);
                          setSidebarToast(`已移动到 ${project.name}`);
                        }}
                        canDelete={sessions.length > 1}
                      />
                    ))}
                </div>
              </div>
            )}

            {/* Category B: Recent (最近) */}
            <div className="px-2">
              <div className="text-[11px] font-semibold text-zinc-500 px-2.5 mb-1 flex items-center gap-1.5 uppercase tracking-wider">
                {t("recent")}
              </div>
              {recentSessions.length === 0 ? (
                <div className="text-xs text-zinc-600 px-3 py-1">
                  {t("noMatchingChats")}
                </div>
              ) : (
                <div className="space-y-0.5">
                  {recentSessions
                    .sort((a, b) => b.updatedAt - a.updatedAt)
                    .map((session) => (
                      <SidebarSessionItem
                        key={session.id}
                        session={session}
                        currentSessionId={!activeProject ? currentSessionId : ""}
                        isPinned={false}
                        activeDropdown={activeDropdown}
                        dropdownPos={dropdownPos}
                        onSelectSession={onSelectSession}
                        togglePin={togglePin}
                        onDeleteSession={onDeleteSession}
                        onRenameSession={onRenameSession}
                        handleDropdownClick={handleDropdownClick}
                        setActiveDropdown={setActiveDropdown}
                        t={t}
                        projectsList={projects}
                        onMoveToProject={(project) => {
                          void onMoveSessionToProject(session.id, project);
                          setSidebarToast(`已移动到 ${project.name}`);
                        }}
                        canDelete={sessions.length > 1}
                      />
                    ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
      <BirdCoderModal
        isOpen={isBirdCoderModalOpen}
        onClose={() => setIsBirdCoderModalOpen(false)}
      />
      <SearchModal
        isOpen={isSearchModalOpen}
        onClose={() => setIsSearchModalOpen(false)}
        sessions={sessions}
        onSelectSession={onSelectSession}
      />
    </>
  );
};
