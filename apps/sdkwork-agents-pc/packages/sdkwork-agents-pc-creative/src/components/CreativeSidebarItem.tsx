import React from 'react';
import { MessageSquare, Pin, MoreHorizontal, Edit3, Trash2 } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { CreativeSession } from '../types';

interface CreativeSidebarItemProps {
  session: CreativeSession;
  activeSessionId: string;
  setActiveSessionId: (id: string) => void;
  editingSessionId: string | null;
  setEditingSessionId: (id: string | null) => void;
  editingTitle: string;
  setEditingTitle: (title: string) => void;
  onSaveRename: (id: string) => void;
  activeSessionMenuId: string | null;
  setActiveSessionMenuId: (id: string | null) => void;
  handleTogglePin: (id: string) => void;
  handleDeleteSession: (e: React.MouseEvent, id: string) => void;
  isDefault?: boolean;
}

export const CreativeSidebarItem: React.FC<CreativeSidebarItemProps> = ({
  session,
  activeSessionId,
  setActiveSessionId,
  editingSessionId,
  setEditingSessionId,
  editingTitle,
  setEditingTitle,
  onSaveRename,
  activeSessionMenuId,
  setActiveSessionMenuId,
  handleTogglePin,
  handleDeleteSession,
  isDefault
}) => {
  return (
    <div 
      onClick={() => setActiveSessionId(session.id)}
      className={cn(
        "w-full flex items-center justify-between px-3 py-2.5 rounded-xl transition-all text-left group cursor-pointer select-none relative",
        editingSessionId === session.id 
          ? "text-zinc-900 dark:text-white" 
          : activeSessionId === session.id 
            ? "bg-zinc-100 text-zinc-900 dark:bg-[#333333] dark:text-white" 
            : "hover:bg-black/5 text-zinc-500 hover:text-zinc-800 dark:hover:bg-[#2a2a2a] dark:text-zinc-400 dark:hover:text-zinc-200"
      )}
    >
      <div className="flex items-center gap-3 min-w-0 flex-1">
        {!isDefault && session.avatarUrl ? (
          <img 
            src={session.avatarUrl} 
            className="w-[18px] h-[18px] rounded-full object-cover shrink-0 border border-black/10 dark:border-white/10" 
            alt="" 
            referrerPolicy="no-referrer"
          />
        ) : (
          <MessageSquare size={16} className={cn("shrink-0", session.isPinned ? "text-cyan-400" : "text-zinc-400")} />
        )}
        {editingSessionId === session.id ? (
          <input
            type="text"
            value={editingTitle}
            onChange={(e) => setEditingTitle(e.target.value)}
            onClick={(e) => e.stopPropagation()}
            onBlur={() => onSaveRename(session.id)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') onSaveRename(session.id);
              if (e.key === 'Escape') setEditingSessionId(null);
            }}
            className="bg-zinc-100 text-zinc-900 text-xs px-1.5 py-0.5 rounded border border-zinc-200 outline-none ring-0 w-full focus:outline-none focus:ring-0 dark:bg-[#242426] dark:text-white dark:border-transparent"
            autoFocus
          />
        ) : (
          <span className="text-[14px] font-medium truncate flex-1">{session.title}</span>
        )}
        {session.isPinned && !editingSessionId && (
          <Pin size={10} className="text-cyan-400 shrink-0 rotate-45" />
        )}
      </div>
      {!editingSessionId && (
        <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
          <button 
            onClick={(e) => {
              e.stopPropagation();
              setActiveSessionMenuId(activeSessionMenuId === session.id ? null : session.id);
            }}
            className="p-1 rounded hover:bg-black/5 text-zinc-500 hover:text-zinc-800 dark:hover:bg-[#333333] dark:text-zinc-400 dark:hover:text-zinc-200 cursor-pointer"
          >
            <MoreHorizontal size={14} />
          </button>
        </div>
      )}

      {/* Context menu dropdown */}
      {activeSessionMenuId === session.id && (
        <>
          <div className="fixed inset-0 z-40" onClick={(e) => { e.stopPropagation(); setActiveSessionMenuId(null); }} />
          <div className="absolute right-2 top-10 w-36 bg-white border border-black/10 rounded-xl py-1.5 shadow-2xl z-50 animate-in fade-in duration-100 dark:bg-[#1a1a1c] dark:border-white/10">
            <button
              onClick={(e) => {
                e.stopPropagation();
                setEditingSessionId(session.id);
                setEditingTitle(session.title);
                setActiveSessionMenuId(null);
              }}
              className="w-full text-left px-3 py-1.5 hover:bg-black/5 text-zinc-600 hover:text-zinc-900 text-xs flex items-center gap-2 transition-colors cursor-pointer dark:hover:bg-[#2a2a2a] dark:text-zinc-300 dark:hover:text-white"
            >
              <Edit3 size={12} className="text-zinc-400 dark:text-zinc-500" />
              重命名 / Rename
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleTogglePin(session.id);
                setActiveSessionMenuId(null);
              }}
              className="w-full text-left px-3 py-1.5 hover:bg-black/5 text-zinc-600 hover:text-zinc-900 text-xs flex items-center gap-2 transition-colors cursor-pointer dark:hover:bg-[#2a2a2a] dark:text-zinc-300 dark:hover:text-white"
            >
              <Pin size={12} className={cn("text-zinc-400 dark:text-zinc-500", session.isPinned ? "rotate-45 text-cyan-500 dark:text-cyan-400" : "")} />
              {session.isPinned ? '取消置顶' : '置顶会话'}
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleDeleteSession(e, session.id);
                setActiveSessionMenuId(null);
              }}
              className="w-full text-left px-3 py-1.5 hover:bg-rose-50 text-rose-600 hover:text-rose-700 text-xs flex items-center gap-2 transition-colors cursor-pointer dark:hover:bg-zinc-900 dark:text-rose-400 dark:hover:text-rose-300"
            >
              <Trash2 size={12} />
              {isDefault ? '重置会话' : '删除会话'}
            </button>
          </div>
        </>
      )}
    </div>
  );
};
