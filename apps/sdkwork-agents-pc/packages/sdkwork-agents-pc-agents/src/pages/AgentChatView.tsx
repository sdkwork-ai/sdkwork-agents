import React, { useEffect, useRef, useState } from "react";
import { motion } from "motion/react";
import { ChevronLeft, Bot, Copy, Check } from "lucide-react";

import { Avatar } from "@sdkwork/agents-pc-commons";

import { LazyMessageInput } from "../components/LazyMessageInput";
import { toast } from "../components/Toast";
import { agentService } from "../services/AgentService";
import { agentChatService, type ChatMessage } from "../services/AgentChatService";
import { createDefaultAvatar } from "../services/DefaultAvatarService";
import type { AgentsDriveMediaResource } from "@sdkwork/agents-pc-core/sdk/driveUploadService";

export interface AgentChatViewProps {
  agentId: string;
  agentName?: string;
  welcomeMessage?: string;
  onBack: () => void;
}

/** Maximum in-memory chat bubbles for one interactive session view. */
const MAX_CHAT_MESSAGES = 200;

function trimMessages(messages: ChatMessage[]): ChatMessage[] {
  if (messages.length <= MAX_CHAT_MESSAGES) {
    return messages;
  }
  return messages.slice(messages.length - MAX_CHAT_MESSAGES);
}

function formatTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

export const AgentChatView: React.FC<AgentChatViewProps> = ({
  agentId,
  agentName: initialAgentName,
  welcomeMessage: initialWelcomeMessage,
  onBack,
}) => {
  const [agentName, setAgentName] = useState(initialAgentName ?? "智能体");
  const [welcomeMessage, setWelcomeMessage] = useState(
    initialWelcomeMessage ?? "你好，我是你的智能助手，有什么可以帮你的？",
  );
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [nextOlderCursor, setNextOlderCursor] = useState<string | undefined>();
  const [hasOlderMessages, setHasOlderMessages] = useState(false);
  const [loadingOlder, setLoadingOlder] = useState(false);
  const [isTyping, setIsTyping] = useState(false);
  const [bootstrapping, setBootstrapping] = useState(true);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;

    const bootstrap = async () => {
      setBootstrapping(true);
      try {
        let sessionTitle = initialAgentName ?? "Chat";
        if (!initialAgentName || !initialWelcomeMessage) {
          const agent = await agentService.getAgent(agentId);
          if (agent && !cancelled) {
            sessionTitle = agent.name;
            setAgentName(agent.name);
            if (agent.welcomeMessage) {
              setWelcomeMessage(agent.welcomeMessage);
            }
          }
        }

        const resolvedSessionId = await agentChatService.resolveOrCreateSession(agentId, sessionTitle);
        if (cancelled) {
          return;
        }
        setSessionId(resolvedSessionId);

        const historyPage = await agentChatService.loadRecentMessages(agentId, resolvedSessionId);
        if (!cancelled) {
          setMessages(trimMessages(historyPage.items));
          setNextOlderCursor(historyPage.pageInfo.nextCursor);
          setHasOlderMessages(historyPage.pageInfo.hasMore);
        }
      } catch {
        if (!cancelled) {
          toast("无法启动聊天会话，请检查登录与后端连接", "error");
        }
      } finally {
        if (!cancelled) {
          setBootstrapping(false);
        }
      }
    };

    void bootstrap();
    return () => {
      cancelled = true;
    };
  }, [agentId, initialAgentName, initialWelcomeMessage]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isTyping]);

  const loadOlderMessages = async () => {
    if (!sessionId || loadingOlder || !hasOlderMessages || !nextOlderCursor) {
      return;
    }

    const container = scrollContainerRef.current;
    const previousScrollHeight = container?.scrollHeight ?? 0;
    const previousScrollTop = container?.scrollTop ?? 0;

    setLoadingOlder(true);
    try {
      const olderPage = await agentChatService.listMessagesPage(
        agentId,
        sessionId,
        nextOlderCursor,
      );
      setMessages((prev) => trimMessages([...olderPage.items, ...prev]));
      setNextOlderCursor(olderPage.pageInfo.nextCursor);
      setHasOlderMessages(olderPage.pageInfo.hasMore);
      requestAnimationFrame(() => {
        if (!container) {
          return;
        }
        container.scrollTop = container.scrollHeight - previousScrollHeight + previousScrollTop;
      });
    } catch {
      toast("无法加载更早的消息", "error");
    } finally {
      setLoadingOlder(false);
    }
  };

  const handleScroll = (event: React.UIEvent<HTMLDivElement>) => {
    if (event.currentTarget.scrollTop <= 48) {
      void loadOlderMessages();
    }
  };

  const handleSend = async (
    content: string,
    _type: 'text' | 'image' | 'file' | 'voice' | 'video' = 'text',
    media?: AgentsDriveMediaResource,
  ) => {
    if (!content.trim() || isTyping || !sessionId) {
      return;
    }

    const userMessage: ChatMessage = {
      id: `local-user-${Date.now()}`,
      role: "user",
      content: media?.fileName ?? content.trim(),
      createdAt: new Date().toISOString(),
    };
    setMessages((prev) => trimMessages([...prev, userMessage]));
    setIsTyping(true);

    try {
      const assistant = await agentChatService.sendMessage(
        agentId,
        sessionId,
        content.trim(),
        undefined,
        media,
      );
      setMessages((prev) => trimMessages([...prev, assistant]));
    } catch {
      setMessages((prev) => prev.filter((message) => message.id !== userMessage.id));
      toast("发送失败：后端未返回有效回复", "error");
    } finally {
      setIsTyping(false);
    }
  };

  const handleCopy = (id: string, content: string) => {
    void navigator.clipboard.writeText(content);
    setCopiedId(id);
    window.setTimeout(() => setCopiedId(null), 2000);
  };

  const visibleMessages =
    messages.length > 0
      ? messages
      : [
          {
            id: "welcome",
            role: "assistant" as const,
            content: welcomeMessage,
            createdAt: new Date().toISOString(),
          },
        ];

  return (
    <motion.div
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className="flex min-h-0 flex-1 flex-col bg-[#1e1e1e] text-gray-200"
    >
      <div className="flex h-14 shrink-0 items-center justify-between border-b border-white/5 bg-[#202020] px-4">
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={onBack}
            className="rounded-lg p-2 text-gray-400 transition-colors hover:bg-white/5 hover:text-white"
          >
            <ChevronLeft size={18} />
          </button>
          <Avatar src={createDefaultAvatar("agent")} size="md" fallback="A" />
          <div>
            <h2 className="text-sm font-semibold text-white">{agentName}</h2>
            <p className="text-xs text-gray-500">生产会话 · sessions API</p>
          </div>
        </div>
      </div>

      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        className="custom-scrollbar min-h-0 flex-1 overflow-y-auto px-4 py-4"
      >
        {bootstrapping ? (
          <div className="py-8 text-center text-sm text-gray-500">正在创建会话...</div>
        ) : (
          <div className="mx-auto flex max-w-3xl flex-col gap-4">
            {hasOlderMessages ? (
              <div className="py-2 text-center text-xs text-gray-500">
                {loadingOlder ? "正在加载更早的消息..." : "向上滚动加载更早的消息"}
              </div>
            ) : null}
            {visibleMessages.map((message) => {
              const isUser = message.role === "user";
              return (
                <div
                  key={message.id}
                  className={`flex ${isUser ? "justify-end" : "justify-start"}`}
                >
                  <div
                    className={`group relative max-w-[85%] rounded-2xl px-4 py-3 text-sm leading-relaxed ${
                      isUser
                        ? "bg-purple-600/90 text-white"
                        : "border border-white/5 bg-[#262626] text-gray-100"
                    }`}
                  >
                    <p className="whitespace-pre-wrap">{message.content}</p>
                    <div className="mt-2 flex items-center justify-between gap-3 text-[11px] text-gray-400">
                      <span>{formatTime(message.createdAt)}</span>
                      {!isUser ? (
                        <button
                          type="button"
                          onClick={() => handleCopy(message.id, message.content)}
                          className="opacity-0 transition-opacity group-hover:opacity-100"
                        >
                          {copiedId === message.id ? <Check size={12} /> : <Copy size={12} />}
                        </button>
                      ) : null}
                    </div>
                  </div>
                </div>
              );
            })}
            {isTyping ? (
              <div className="text-sm text-gray-500">智能体正在回复...</div>
            ) : null}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      <div className="shrink-0 border-t border-white/5 bg-[#202020] p-3">
        <LazyMessageInput
          onSend={handleSend}
          uploadResourceId={`${agentId}:${sessionId ?? 'pending'}`}
          placeholder="输入消息，使用生产 sessions/messages API..."
          disabled={bootstrapping || !sessionId}
          isTyping={isTyping}
        />
      </div>
    </motion.div>
  );
};
