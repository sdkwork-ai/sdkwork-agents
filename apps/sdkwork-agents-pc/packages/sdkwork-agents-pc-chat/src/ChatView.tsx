import React, { useState, useRef, useEffect, useCallback, useMemo } from "react";
import {
  ChatMessage,
  ChatSession,
  type ChatProject,
} from "@sdkwork/agents-pc-chat";
import { cn } from "@sdkwork/agents-pc-commons";
import { ChatService } from "./services/ChatService";
import { ProjectService } from "./services/ProjectService";
import { useTranslation } from "react-i18next";
import {
  agentsDriveUploadService,
  type AgentsDriveMediaResource,
  type AgentsDriveUploadPurpose,
} from "@sdkwork/agents-pc-core/sdk/driveUploadService";
import { uuid } from "@sdkwork/utils";

import { Sidebar } from "./components/Sidebar";
import { ChatHeader } from "./components/ChatHeader";
import { MessageList } from "./components/MessageList";
import { ChatInput } from "./components/ChatInput";
import { ArtifactPanel } from "./components/ArtifactPanel";
import { SettingsModal } from "./components/SettingsModal";
import { CustomProviderDialog } from "./components/CustomProviderDialog";
import { UserProfileModal } from "./components/UserProfileModal";
import { ProjectHomeView } from "./components/ProjectHomeView";
import { ProjectSettingsModal } from "./components/ProjectSettingsModal";
import { FileLibraryView } from "./components/FileLibraryView";
import {
  chatModelPickerGroups,
  createChatModelPickerFallback,
  createCustomProviderModelGroup,
  resolveChatDefaultModelId,
} from "./modelPicker/chatModelPickerCatalog";
import type { ModelsPickerGroup } from "@sdkwork/models-pc-picker/model-picker-types";
import type { AppliedCustomProvider } from "./components/CustomProviderDialog";

const chatModelPickerFallback = createChatModelPickerFallback();

function resolveChatUploadPurpose(file: File): AgentsDriveUploadPurpose {
  if (file.type.startsWith('image/')) return 'agent-chat-image';
  if (file.type.startsWith('video/')) return 'agent-chat-video';
  if (file.type.startsWith('audio/')) return 'agent-chat-voice';
  return 'agent-chat-attachment';
}

export const ChatView = () => {
  const { t, i18n } = useTranslation("chat");

  const [sessions, setSessions] = useState<ChatSession[]>([
    { id: "1", title: t("newChat"), messages: [], updatedAt: Date.now(), version: "" },
  ]);
  const [currentSessionId, setCurrentSessionId] = useState<string>("1");
  const [activeView, setActiveView] = useState<'chat' | 'library'>('chat');
  const [projects, setProjects] = useState<ChatProject[]>([]);
  const [activeProject, setActiveProject] = useState<ChatProject | null>(null);
  const [activeProjectSettings, setActiveProjectSettings] = useState<
    ChatProject | null
  >(null);
  const [input, setInput] = useState(() => {
    try {
      return localStorage.getItem("chat_input_draft") || "";
    } catch {
      return "";
    }
  });

  useEffect(() => {
    void ProjectService.getProjects()
      .then(setProjects)
      .catch((error) => console.error("Project list failed", error));
  }, []);

  useEffect(() => {
    let active = true;
    void ChatService.loadSessions(selectedModel)
      .then((remoteSessions) => {
        if (!active || remoteSessions.length === 0) return;
        setSessions(remoteSessions);
        setCurrentSessionId(remoteSessions[0].id);
        // Lazy detail: load the transcript of the initially selected session.
        void ChatService.loadSessionDetail(remoteSessions[0].id)
          .then((messages) => {
            if (!active) return;
            setSessions((prev) =>
              prev.map((s) => s.id === remoteSessions[0].id ? { ...s, messages } : s),
            );
          })
          .catch((error) => console.error("Chat transcript load failed", error));
      })
      .catch((error) => console.error("Chat history load failed", error));
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    try {
      localStorage.setItem("chat_input_draft", input);
    } catch {
      // ignore
    }
  }, [input]);
  const [selectedMediaResources, setSelectedMediaResources] = useState<AgentsDriveMediaResource[]>([]);
  // Image previews are derived from the selected media so file attachments
  // never leak into the image preview strip.
  const selectedImages = useMemo(
    () => selectedMediaResources
      .filter((media) => media.kind === 'image')
      .flatMap((media) => (media.url ? [media.url] : [])),
    [selectedMediaResources],
  );
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [isGenerating, setIsGenerating] = useState(false);

  const [selectedModel, setSelectedModel] = useState(resolveChatDefaultModelId);
  const [isModelSelectorOpen, setIsModelSelectorOpen] = useState(false);
  const [isCustomProviderOpen, setIsCustomProviderOpen] = useState(false);
  const [customProviderGroups, setCustomProviderGroups] = useState<ModelsPickerGroup[]>([]);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [profileModalTab, setProfileModalTab] = useState<
    "profile" | "billing" | null
  >(null);
  const [shouldAutoScroll, setShouldAutoScroll] = useState(true);

  const [artifact, setArtifact] = useState<{
    language: string;
    code: string;
    mode: "preview" | "code";
  } | null>(null);
  const [isArtifactOpen, setIsArtifactOpen] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const abortControllerRef = useRef<AbortController | null>(null);
  const pinMutationsRef = useRef<Set<string>>(new Set());
  const feedbackMutationsRef = useRef<Set<string>>(new Set());

  const currentSession = sessions.find((s) => s.id === currentSessionId)!;

  const handleScroll = () => {
    if (!scrollContainerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } =
      scrollContainerRef.current;
    const isAtBottom = scrollHeight - scrollTop - clientHeight <= 100;
    setShouldAutoScroll(isAtBottom);
  };

  useEffect(() => {
    if (sessions.length === 1 && sessions[0].messages.length === 0 && sessions[0].title !== t("newChat")) {
      setSessions((prev) => [{ ...prev[0], title: t("newChat") }]);
    }
  }, [t, sessions]);

  useEffect(() => {
    const scrollContainer = scrollContainerRef.current;
    if (!scrollContainer) return;

    const observer = new MutationObserver(() => {
      if (shouldAutoScroll) {
        messagesEndRef.current?.scrollIntoView({ behavior: "auto" });
      }
    });

    observer.observe(scrollContainer, {
      childList: true,
      subtree: true,
      characterData: true
    });

    return () => observer.disconnect();
  }, [shouldAutoScroll]);

  useEffect(() => {
    if (shouldAutoScroll) {
      messagesEndRef.current?.scrollIntoView({ behavior: "auto" });
    }
  }, [currentSession?.messages, shouldAutoScroll, isGenerating]);

  // Parse for artifacts when generation completes
  useEffect(() => {
    if (!isGenerating && currentSession?.messages?.length > 0) {
      const lastMessage =
        currentSession.messages[currentSession.messages.length - 1];
      if (lastMessage.role === "model" && lastMessage.text) {
        const regex =
          /```(html|js|javascript|jsx|ts|typescript|tsx|css|json|markdown|md|svg|xml)\n([\s\S]*?)```/g;
        let lastMatch;
        let match;
        while ((match = regex.exec(lastMessage.text)) !== null) {
          lastMatch = match;
        }
        if (lastMatch) {
          const lang = lastMatch[1].toLowerCase();
          const code = lastMatch[2];
          setArtifact({
            language: lang,
            code,
            mode: ["html", "svg", "xml"].includes(lang) ? "preview" : "code",
          });
          setIsArtifactOpen(true);
        }
      }
    }
  }, [isGenerating, currentSession?.id]);

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  const handleNewChat = useCallback(() => {
    const newSession: ChatSession = {
      id: uuid(),
      title: t("newChat"),
      messages: [],
      updatedAt: Date.now(),
      version: "",
    };
    setSessions((prev) => [newSession, ...prev]);
    setCurrentSessionId(newSession.id);
    setActiveProject(null);
    setActiveView('chat');
    setIsArtifactOpen(false);
    setShouldAutoScroll(true);
    setSelectedMediaResources([]);
  }, [t]);

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        if (e.key.toLowerCase() === 'n') {
          e.preventDefault();
          handleNewChat();
        } else if (e.key.toLowerCase() === 'b') {
          e.preventDefault();
          setIsSidebarOpen(prev => !prev);
        } else if (e.key.toLowerCase() === 'f') {
          e.preventDefault();
          setIsSidebarOpen(true);
          setTimeout(() => {
            document.getElementById('sidebar-search-input')?.focus();
          }, 100);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleNewChat]);

  const handleRenameSession = async (id: string, newTitle: string) => {
    const session = sessions.find((item) => item.id === id);
    if (!session) return;
    const updated = await ChatService.renameSession(id, newTitle, session.version);
    setSessions((prev) => prev.map((item) => item.id === id ? {
      ...item,
      title: updated.title,
      version: updated.version,
      updatedAt: Date.parse(updated.updatedAt) || Date.now(),
    } : item));
  };

  const handleTogglePin = async (id: string, pinned: boolean): Promise<boolean> => {
    if (pinMutationsRef.current.has(id)) return false;
    const session = sessions.find((item) => item.id === id);
    if (!session) return false;
    pinMutationsRef.current.add(id);
    setSessions((previous) => previous.map((item) =>
      item.id === id ? { ...item, pinned } : item
    ));
    try {
      const updated = await ChatService.setSessionPinned(
        id,
        pinned,
        session.userStateVersion,
      );
      setSessions((previous) => previous.map((item) => item.id === id ? {
        ...item,
        pinned: updated.pinned,
        userStateVersion: updated.version,
      } : item));
      return true;
    } catch (error) {
      setSessions((previous) => previous.map((item) => item.id === id ? {
        ...item,
        pinned: session.pinned ?? false,
        userStateVersion: session.userStateVersion,
      } : item));
      console.error("Chat pin update failed", error);
      throw error;
    } finally {
      pinMutationsRef.current.delete(id);
    }
  };

  const handleMessageFeedback = async (
    messageId: string,
    rating: 'up' | 'down' | undefined,
  ): Promise<boolean> => {
    const mutationKey = `${currentSessionId}:${messageId}`;
    if (feedbackMutationsRef.current.has(mutationKey)) return false;
    const session = sessions.find((item) => item.id === currentSessionId);
    const message = session?.messages.find((item) => item.id === messageId);
    if (!session || !message || message.role !== 'model') return false;
    feedbackMutationsRef.current.add(mutationKey);
    setSessions((previous) => previous.map((item) => item.id === session.id ? {
      ...item,
      messages: item.messages.map((candidate) =>
        candidate.id === messageId ? { ...candidate, feedback: rating } : candidate
      ),
    } : item));
    try {
      const updated = await ChatService.setMessageFeedback(
        session.id,
        messageId,
        rating,
        message.feedbackVersion,
      );
      setSessions((previous) => previous.map((item) => item.id === session.id ? {
        ...item,
        messages: item.messages.map((candidate) => candidate.id === messageId ? {
          ...candidate,
          feedback: updated.rating,
          feedbackVersion: updated.version,
        } : candidate),
      } : item));
      return true;
    } catch (error) {
      setSessions((previous) => previous.map((item) => item.id === session.id ? {
        ...item,
        messages: item.messages.map((candidate) => candidate.id === messageId ? {
          ...candidate,
          feedback: message.feedback,
          feedbackVersion: message.feedbackVersion,
        } : candidate),
      } : item));
      console.error("Chat message feedback update failed", error);
      throw error;
    } finally {
      feedbackMutationsRef.current.delete(mutationKey);
    }
  };

  const handleCreateProject = async (title: string) => {
    const created = await ProjectService.createProject(title);
    setProjects((previous) => [created, ...previous]);
  };

  const handleRenameProject = async (project: ChatProject, newTitle: string) => {
    const updated = await ProjectService.updateProject(project, { name: newTitle });
    setProjects((previous) =>
      previous.map((item) => item.projectId === project.projectId ? updated : item),
    );
    if (activeProject?.projectId === project.projectId) setActiveProject(updated);
  };

  const handleDeleteProject = async (project: ChatProject) => {
    await ProjectService.deleteProject(project.projectId);
    setProjects((previous) =>
      previous.filter((item) => item.projectId !== project.projectId),
    );
    if (activeProject?.projectId === project.projectId) setActiveProject(null);
  };

  const handleSelectSession = (id: string) => {
    setCurrentSessionId(id);
    setActiveProject(null);
    setActiveView('chat');
    setIsArtifactOpen(false);
    setShouldAutoScroll(true);
    // Draft media belongs to the conversation it was selected in; never
    // carry attachments into another session.
    setSelectedMediaResources([]);
    // Lazy detail: transcripts are not preloaded with the session list, so
    // fetch the selected session's messages when they are not loaded yet.
    const session = sessions.find((s) => s.id === id);
    if (session && session.messages.length === 0) {
      void ChatService.loadSessionDetail(id)
        .then((messages) => {
          setSessions((prev) =>
            prev.map((s) => (s.id === id ? { ...s, messages } : s)),
          );
        })
        .catch((error) => console.error("Chat transcript load failed", error));
    }
  };

  const handleDeleteSession = async (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    await ChatService.deleteSession(id);
    setSessions((prev) => prev.filter((s) => s.id !== id));
    if (currentSessionId === id) {
      setCurrentSessionId(sessions.find((s) => s.id !== id)?.id || "");
      setSelectedMediaResources([]);
    }
  };

  const handleMoveSessionToProject = async (sessionId: string, project: ChatProject) => {
    const session = sessions.find((item) => item.id === sessionId);
    if (!session) return;
    const updated = await ChatService.moveSession(sessionId, project.projectId, session.version);
    setSessions((previous) => previous.map((item) => item.id === sessionId ? {
      ...item,
      projectId: project.projectId,
      version: updated.version,
      updatedAt: Date.parse(updated.updatedAt) || Date.now(),
    } : item));
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files: File[] = [];
    for (let index = 0; index < (event.target.files?.length ?? 0); index += 1) {
      const file = event.target.files?.item(index);
      if (file) files.push(file);
    }
    event.target.value = '';
    if (files.length === 0) return;

    // Upload each file independently so a rejected file (e.g. over the size
    // limit) never discards the rest of the selection.
    files.forEach((file) => {
      void agentsDriveUploadService.upload({
        file,
        purpose: resolveChatUploadPurpose(file),
        resourceId: `agents-chat:${currentSessionId}`,
      })
        .then((media) => {
          setSelectedMediaResources((previous) => [...previous, media]);
        })
        .catch((error) => {
          console.error(`Chat file Drive upload failed for ${file.name}`, error);
          window.alert(`${file.name} 上传失败，请重试`);
        });
    });
  };

  const handleRemoveImage = (index: number) => {
    // The preview strip only shows image media, so map the preview index back
    // to the media resource before removing it.
    setSelectedMediaResources((previous) => {
      const imageMedias = previous.filter((media) => media.kind === 'image');
      const target = imageMedias[index];
      if (!target) return previous;
      return previous.filter((media) => media.id !== target.id);
    });
  };

  const handleSend = async () => {
    if ((!input.trim() && selectedMediaResources.length === 0) || isGenerating) return;

    setShouldAutoScroll(true);

    const userMessage: ChatMessage = {
      id: uuid(),
      role: "user",
      text: input.trim(),
      images: selectedImages.length > 0 ? [...selectedImages] : undefined,
      mediaResources: selectedMediaResources.length > 0 ? [...selectedMediaResources] : undefined,
    };

    setShouldAutoScroll(true);
    setInput("");
    setSelectedMediaResources([]);
    setIsGenerating(true);

    // Add user message
    setSessions((prev) =>
      prev.map((s) => {
        if (s.id === currentSessionId) {
          // Auto-generate title for first message
          const titleText = userMessage.text || t("imageChat");
          const title =
            s.messages.length === 0
              ? titleText.slice(0, 30) + (titleText.length > 30 ? "..." : "")
              : s.title;
          return {
            ...s,
            title,
            updatedAt: Date.now(),
            messages: [...s.messages, userMessage],
          };
        }
        return s;
      }),
    );

    // Add empty model message placeholder
    const modelMessageId = uuid();
    setSessions((prev) =>
      prev.map((s) =>
        s.id === currentSessionId
          ? {
              ...s,
              messages: [
                ...s.messages,
                { id: modelMessageId, role: "model", text: "" },
              ],
            }
          : s,
      ),
    );

    const updatedSession = sessions.find((s) => s.id === currentSessionId)!;
    const requestMessages = updatedSession.messages.concat(userMessage);

    // Abort previous generation if exist
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    const abortController = new AbortController();
    abortControllerRef.current = abortController;

    try {
      await ChatService.streamChat({
        sessionId: currentSessionId,
        model: selectedModel,
        messages: requestMessages,
        signal: abortController.signal,
        onMessageUpdate: (text) => {
          setSessions((prev) =>
            prev.map((s) => {
              if (s.id === currentSessionId) {
                return {
                  ...s,
                  messages: s.messages.map((m) =>
                    m.id === modelMessageId ? { ...m, text: m.text + text } : m,
                  ),
                };
              }
              return s;
            }),
          );
        },
        onComplete: (completedMessage) => {
          if (completedMessage?.id) {
            setSessions((previous) => previous.map((session) =>
              session.id === currentSessionId
                ? {
                    ...session,
                    messages: session.messages.map((message) =>
                      message.id === modelMessageId
                        ? { ...message, id: completedMessage.id }
                        : message
                    ),
                  }
                : session
            ));
          }
          setIsGenerating(false);
          abortControllerRef.current = null;
        },
        onError: (failure) => {
          if (failure.message !== "AbortError") {
            // Map the backend problem to localized text per FRONTEND_SPEC:
            // prefer the explicit `i18nKey`, fall back to the standard
            // `errors.result.<code>` key, and finally to a clean generic
            // message (the raw SDK text stays in the console log).
            const mappedKey =
              failure.i18nKey
              ?? (failure.code !== undefined ? `errors.result.${failure.code}` : undefined);
            const translated =
              mappedKey && i18n.exists(mappedKey)
                ? String(i18n.t(mappedKey))
                : t("sendErrorFallback");
            const hint =
              failure.httpStatus !== undefined && failure.httpStatus >= 500
                ? t("retryHint")
                : "";
            const errorText = `${translated}${hint}`.trim();
            setSessions((prev) =>
              prev.map((s) => {
                if (s.id === currentSessionId) {
                  return {
                    ...s,
                    messages: s.messages.map((m) =>
                      m.id === modelMessageId
                        ? {
                            ...m,
                            text: m.text + `\n\n**⚠️ ${errorText}**`,
                          }
                        : m,
                    ),
                  };
                }
                return s;
              }),
            );
          }
          setIsGenerating(false);
          abortControllerRef.current = null;
        },
      });
    } catch (e: any) {
      if (e.name !== "AbortError") {
        setIsGenerating(false);
        abortControllerRef.current = null;
      }
    }
  };

  const handleStop = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
      setIsGenerating(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex h-full w-full bg-[#f5f5f5] dark:bg-[#191919] font-sans text-gray-900 dark:text-gray-100 overflow-hidden">
      <Sidebar
        isOpen={isSidebarOpen}
        toggleSidebar={() => setIsSidebarOpen(!isSidebarOpen)}
        sessions={sessions}
        currentSessionId={currentSessionId}
        onNewChat={handleNewChat}
        onSelectSession={handleSelectSession}
        onDeleteSession={handleDeleteSession}
        onRenameSession={handleRenameSession}
        onTogglePin={handleTogglePin}
        onMoveSessionToProject={handleMoveSessionToProject}
        onOpenUserProfile={(tab) => setProfileModalTab(tab)}
        onOpenFileLibrary={() => setActiveView('library')}
        projects={projects}
        activeProject={activeProject?.projectId}
        onProjectSelect={setActiveProject}
        onProjectSettings={(project) => setActiveProjectSettings(project)}
        onProjectCreate={handleCreateProject}
        onProjectRename={handleRenameProject}
        onProjectDelete={handleDeleteProject}
      />

      <div
        className={cn(
          "flex flex-col h-full relative bg-[#f5f5f5] dark:bg-[#191919] transition-all duration-300",
          isArtifactOpen
            ? "flex-1 min-w-[300px] border-r border-[#d9d9d9] dark:border-[#1a1a1a]"
            : "flex-1 min-w-0",
        )}
      >
        {activeView === 'library' ? (
          <FileLibraryView />
        ) : !activeProject ? (
          <>
            <ChatHeader
              isSidebarOpen={isSidebarOpen}
              toggleSidebar={() => setIsSidebarOpen(!isSidebarOpen)}
              modelGroups={[...chatModelPickerGroups, ...customProviderGroups]}
              selectedModelId={selectedModel}
              onSelectModel={setSelectedModel}
              fallbackModel={chatModelPickerFallback}
              isModelSelectorOpen={isModelSelectorOpen}
              setIsModelSelectorOpen={setIsModelSelectorOpen}
              onOpenSettings={() => setIsSettingsOpen(true)}
              onManageCustomProvider={() => setIsCustomProviderOpen(true)}
            />

            <div
              className="flex-1 overflow-y-auto w-full"
              ref={scrollContainerRef}
              onScroll={handleScroll}
            >
              <MessageList
                messages={currentSession?.messages || []}
                messagesEndRef={messagesEndRef}
                onFeedback={handleMessageFeedback}
                onOpenArtifact={(lang, code, mode) => {
                  const finalMode =
                    mode ||
                    (["html", "svg", "xml", "md", "markdown"].includes(
                      lang.toLowerCase(),
                    )
                      ? "preview"
                      : "code");
                  setArtifact({
                    language: lang.toLowerCase(),
                    code,
                    mode: finalMode,
                  });
                  setIsArtifactOpen(true);
                }}
              />
            </div>

            <ChatInput
              input={input}
              setInput={setInput}
              selectedImages={selectedImages}
              isGenerating={isGenerating}
              textareaRef={textareaRef}
              fileInputRef={fileInputRef}
              handleFileChange={handleFileChange}
              handleRemoveImage={handleRemoveImage}
              handleSend={handleSend}
              handleStop={handleStop}
              handleKeyDown={handleKeyDown}
            />
          </>
        ) : (
          <ProjectHomeView projectId={activeProject.projectId} />
        )}
      </div>

      {isArtifactOpen && (
        <ArtifactPanel
          artifact={artifact}
          onClose={() => setIsArtifactOpen(false)}
          onModeChange={(mode) =>
            setArtifact((prev) => (prev ? { ...prev, mode } : null))
          }
          onCodeChange={(code) =>
            setArtifact((prev) => (prev ? { ...prev, code } : null))
          }
        />
      )}

      {isSettingsOpen && (
        <SettingsModal onClose={() => setIsSettingsOpen(false)} />
      )}
      {isCustomProviderOpen && (
        <CustomProviderDialog
          open={isCustomProviderOpen}
          onClose={() => setIsCustomProviderOpen(false)}
          onApplied={(provider: AppliedCustomProvider) => {
            setCustomProviderGroups([
              createCustomProviderModelGroup(provider),
            ]);
            setSelectedModel(provider.modelId);
          }}
        />
      )}
      {activeProjectSettings && (
        <ProjectSettingsModal
          project={activeProjectSettings}
          onClose={() => setActiveProjectSettings(null)}
          onSaved={(updated) => {
            setProjects((previous) => previous.map((item) =>
              item.projectId === updated.projectId ? updated : item
            ));
            if (activeProject?.projectId === updated.projectId) setActiveProject(updated);
            setActiveProjectSettings(updated);
          }}
          onDeleted={(projectId) => {
            setProjects((previous) => previous.filter((item) => item.projectId !== projectId));
            if (activeProject?.projectId === projectId) setActiveProject(null);
            setActiveProjectSettings(null);
          }}
        />
      )}
      {profileModalTab !== null && (
        <UserProfileModal
          initialTab={profileModalTab}
          onClose={() => setProfileModalTab(null)}
        />
      )}
    </div>
  );
};
