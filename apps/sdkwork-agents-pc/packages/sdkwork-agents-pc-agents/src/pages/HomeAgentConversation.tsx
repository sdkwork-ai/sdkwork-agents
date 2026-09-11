import { ArrowLeft, Bot, LoaderCircle, Send, User } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';

import { agentChatService, type ChatMessage } from '../services/AgentChatService';
import type { AgentConfig } from '../services/AgentService';

interface HomeAgentConversationProps {
  agent: AgentConfig;
  agentId: string;
  onBack: () => void;
}

export function HomeAgentConversation({ agent, agentId, onBack }: HomeAgentConversationProps) {
  const [sessionId, setSessionId] = useState<string>();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [draft, setDraft] = useState('');
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string>();
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;

    const loadConversation = async () => {
      setLoading(true);
      setError(undefined);
      try {
        const resolvedSessionId = await agentChatService.resolveOrCreateSession(agentId, agent.name);
        const history = await agentChatService.loadRecentMessages(agentId, resolvedSessionId);
        if (!cancelled) {
          setSessionId(resolvedSessionId);
          setMessages(history.items);
        }
      } catch (loadError) {
        console.error('Failed to load agent conversation.', loadError);
        if (!cancelled) {
          setError('会话暂时无法连接，请稍后重试。');
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };

    void loadConversation();
    return () => {
      cancelled = true;
    };
  }, [agentId, agent.name]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, sending]);

  const sendMessage = async () => {
    const content = draft.trim();
    if (!content || !sessionId || sending) {
      return;
    }

    const pending: ChatMessage = {
      id: `local-${Date.now()}`,
      role: 'user',
      content,
      createdAt: new Date().toISOString(),
    };
    setDraft('');
    setError(undefined);
    setSending(true);
    setMessages((current) => [...current, pending]);
    try {
      const response = await agentChatService.sendMessage(agentId, sessionId, content);
      setMessages((current) => [...current, response]);
    } catch (sendError) {
      console.error('Failed to send agent message.', sendError);
      setMessages((current) => current.filter((message) => message.id !== pending.id));
      setDraft(content);
      const detail = sendError instanceof Error && sendError.message.trim()
        ? sendError.message
        : '';
      setError(detail || '消息发送失败，请重试。');
    } finally {
      setSending(false);
    }
  };

  const visibleMessages = messages.length > 0
    ? messages
    : [{
        id: 'welcome',
        role: 'assistant' as const,
        content: agent.welcomeMessage || `你好，我是 ${agent.name}。`,
        createdAt: new Date().toISOString(),
      }];

  return (
    <div className="flex h-full min-h-0 flex-col bg-[#111113] text-zinc-100">
      <header className="flex h-16 shrink-0 items-center gap-3 border-b border-white/[0.07] px-4 sm:px-6">
        <button aria-label="返回 Agent 列表" className="rounded-lg p-2 text-zinc-500 transition hover:bg-white/[0.06] hover:text-white" onClick={onBack} type="button">
          <ArrowLeft size={19} />
        </button>
        <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-cyan-500/15 text-cyan-300">
          <Bot size={19} />
        </div>
        <div className="min-w-0">
          <h1 className="truncate text-sm font-semibold">{agent.name}</h1>
          <p className="text-xs text-zinc-600">Agent 会话</p>
        </div>
      </header>

      <main className="min-h-0 flex-1 overflow-y-auto px-4 py-6 sm:px-6">
        <div className="mx-auto flex w-full max-w-3xl flex-col gap-4">
          {loading ? (
            <div className="flex min-h-48 items-center justify-center text-zinc-600"><LoaderCircle className="animate-spin" size={22} /></div>
          ) : visibleMessages.map((message) => {
            const fromUser = message.role === 'user';
            return (
              <div className={`flex items-start gap-3 ${fromUser ? 'flex-row-reverse' : ''}`} key={message.id}>
                <div className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-lg ${fromUser ? 'bg-white/[0.07] text-zinc-300' : 'bg-cyan-500/15 text-cyan-300'}`}>
                  {fromUser ? <User size={16} /> : <Bot size={16} />}
                </div>
                <div className={`max-w-[82%] whitespace-pre-wrap rounded-xl px-4 py-3 text-sm leading-6 ${fromUser ? 'bg-cyan-500 text-[#071014]' : 'border border-white/[0.07] bg-white/[0.035] text-zinc-300'}`}>
                  {message.content}
                </div>
              </div>
            );
          })}
          {sending && <div className="flex items-center gap-2 pl-11 text-xs text-zinc-600"><LoaderCircle className="animate-spin" size={14} /> 正在回复</div>}
          <div ref={endRef} />
        </div>
      </main>

      <footer className="shrink-0 border-t border-white/[0.07] bg-[#151517] p-3 sm:p-4">
        <div className="mx-auto flex max-w-3xl items-end gap-2 rounded-xl border border-white/[0.09] bg-black/20 p-2 focus-within:border-cyan-500/40">
          <textarea
            aria-label="消息"
            className="max-h-36 min-h-10 flex-1 resize-none bg-transparent px-2 py-2 text-sm text-zinc-200 outline-none placeholder:text-zinc-700"
            disabled={loading}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && !event.shiftKey) {
                event.preventDefault();
                void sendMessage();
              }
            }}
            placeholder="输入消息"
            rows={1}
            value={draft}
          />
          <button aria-label="发送" className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-cyan-500 text-[#071014] transition hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-40" disabled={!draft.trim() || !sessionId || sending} onClick={() => void sendMessage()} type="button">
            <Send size={17} />
          </button>
        </div>
        {error && <p className="mx-auto mt-2 max-w-3xl px-1 text-xs text-rose-300">{error}</p>}
      </footer>
    </div>
  );
}
