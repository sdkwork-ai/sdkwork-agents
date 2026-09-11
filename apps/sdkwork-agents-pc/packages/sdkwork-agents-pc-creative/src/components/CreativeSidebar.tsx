import React, { useState } from 'react';
import { SidebarClose, SquarePen, History, MessageSquareCode } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { CreativeSession } from '../types';
import { CreativeSidebarItem } from './CreativeSidebarItem';
import { CreativeHistoryLog, PromptHistoryItem } from './CreativeHistoryLog';

interface CreativeSidebarProps {
  isSidebarOpen: boolean;
  setIsSidebarOpen: (isOpen: boolean) => void;
  sessions: CreativeSession[];
  activeSessionId: string;
  setActiveSessionId: (id: string) => void;
  handleNewChat: () => void;
  handleSaveRename: (id: string, customTitle?: string) => void;
  handleTogglePin: (id: string) => void;
  handleDeleteSession: (e: React.MouseEvent, id: string) => void;
  
  // History log props
  historyLogs: PromptHistoryItem[];
  onReRun: (item: PromptHistoryItem) => void;
  onLoadConfig: (item: PromptHistoryItem) => void;
  onDeleteLog: (id: string) => void;
  onClearLogs: () => void;
}

export const CreativeSidebar: React.FC<CreativeSidebarProps> = ({
  isSidebarOpen,
  setIsSidebarOpen,
  sessions,
  activeSessionId,
  setActiveSessionId,
  handleNewChat,
  handleSaveRename,
  handleTogglePin,
  handleDeleteSession,
  
  historyLogs,
  onReRun,
  onLoadConfig,
  onDeleteLog,
  onClearLogs
}) => {
  const [activeSessionMenuId, setActiveSessionMenuId] = useState<string | null>(null);
  const [editingSessionId, setEditingSessionId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState('');
  const [activeTab, setActiveTab] = useState<'sessions' | 'history'>('sessions');

  const onSaveRename = (id: string) => {
    handleSaveRename(id, editingTitle);
    setEditingSessionId(null);
  };

  return (
    <div 
      className={cn(
        "h-full flex flex-col bg-white border-r border-black/5 transition-all duration-300 overflow-hidden shrink-0 dark:bg-[#141414] dark:border-white/5",
        isSidebarOpen ? "w-[260px]" : "w-0 border-none"
      )}
    >
      {/* Sidebar Header */}
      <div className="p-4 flex items-center justify-between h-14 shrink-0">
        <div className="font-medium text-[15px] text-zinc-800 dark:text-zinc-200">开启创作</div>
        <button 
          onClick={() => setIsSidebarOpen(false)}
          className="p-1.5 rounded-lg hover:bg-black/5 text-zinc-500 hover:text-zinc-800 dark:hover:bg-[#333333] dark:text-zinc-400 dark:hover:text-zinc-200 transition-colors cursor-pointer"
        >
          <SidebarClose size={18} />
        </button>
      </div>

      {/* Sidebar Tabs */}
      <div className="px-3 pb-2 flex gap-1.5 shrink-0">
        <button
          onClick={() => setActiveTab('sessions')}
          className={cn(
            "flex-1 py-1.5 rounded-lg text-xs font-semibold flex items-center justify-center gap-1.5 border transition-all cursor-pointer",
            activeTab === 'sessions'
              ? "bg-zinc-100 border-zinc-200 text-zinc-900 shadow-sm dark:bg-[#333333] dark:border-white/10 dark:text-white"
              : "border-transparent text-zinc-500 hover:text-zinc-800 hover:bg-black/5 dark:text-zinc-400 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a]"
          )}
        >
          <MessageSquareCode size={13} />
          <span>创作会话</span>
        </button>
        <button
          onClick={() => setActiveTab('history')}
          className={cn(
            "flex-1 py-1.5 rounded-lg text-xs font-semibold flex items-center justify-center gap-1.5 border transition-all cursor-pointer",
            activeTab === 'history'
              ? "bg-zinc-100 border-zinc-200 text-zinc-900 shadow-sm dark:bg-[#333333] dark:border-white/10 dark:text-white"
              : "border-transparent text-zinc-500 hover:text-zinc-800 hover:bg-black/5 dark:text-zinc-400 dark:hover:text-zinc-200 dark:hover:bg-[#2a2a2a]"
          )}
        >
          <History size={13} />
          <span>历史记录</span>
          {historyLogs.length > 0 && (
            <span className="px-1.5 py-0.2 bg-cyan-500/20 text-cyan-400 rounded-full text-[9px] font-bold">
              {historyLogs.length}
            </span>
          )}
        </button>
      </div>

      {/* Sidebar Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {activeTab === 'sessions' ? (
          <div className="flex-1 overflow-y-auto p-3 space-y-1 custom-scrollbar">
            <button 
              onClick={handleNewChat}
              className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl transition-colors text-left hover:bg-black/5 text-zinc-500 hover:text-zinc-800 dark:hover:bg-[#2a2a2a] dark:text-zinc-400 dark:hover:text-zinc-200 cursor-pointer"
            >
              <SquarePen size={18} />
              <span className="text-[14px] font-medium">新对话</span>
            </button>
            
            <div className="my-2 h-[1px] bg-black/5 dark:bg-[#2f2f2f] mx-2"></div>
            
            {/* Active default creation session */}
            {sessions.filter(s => s.id === 'default').map(session => (
              <CreativeSidebarItem
                key={session.id}
                session={session}
                activeSessionId={activeSessionId}
                setActiveSessionId={setActiveSessionId}
                editingSessionId={editingSessionId}
                setEditingSessionId={setEditingSessionId}
                editingTitle={editingTitle}
                setEditingTitle={setEditingTitle}
                onSaveRename={onSaveRename}
                activeSessionMenuId={activeSessionMenuId}
                setActiveSessionMenuId={setActiveSessionMenuId}
                handleTogglePin={handleTogglePin}
                handleDeleteSession={handleDeleteSession}
                isDefault={true}
              />
            ))}

            {/* Recent list label */}
            {sessions.filter(s => s.id !== 'default').length > 0 && (
              <div className="px-3 pt-4 pb-2 text-[11px] font-medium text-zinc-500 tracking-wider uppercase select-none">
                最近
              </div>
            )}

            {/* Other recent sessions */}
            {sessions.filter(s => s.id !== 'default').map(session => (
              <CreativeSidebarItem
                key={session.id}
                session={session}
                activeSessionId={activeSessionId}
                setActiveSessionId={setActiveSessionId}
                editingSessionId={editingSessionId}
                setEditingSessionId={setEditingSessionId}
                editingTitle={editingTitle}
                setEditingTitle={setEditingTitle}
                onSaveRename={onSaveRename}
                activeSessionMenuId={activeSessionMenuId}
                setActiveSessionMenuId={setActiveSessionMenuId}
                handleTogglePin={handleTogglePin}
                handleDeleteSession={handleDeleteSession}
                isDefault={false}
              />
            ))}
          </div>
        ) : (
          <CreativeHistoryLog
            historyLogs={historyLogs}
            onReRun={onReRun}
            onLoadConfig={onLoadConfig}
            onDeleteLog={onDeleteLog}
            onClearLogs={onClearLogs}
          />
        )}
      </div>
    </div>
  );
};
