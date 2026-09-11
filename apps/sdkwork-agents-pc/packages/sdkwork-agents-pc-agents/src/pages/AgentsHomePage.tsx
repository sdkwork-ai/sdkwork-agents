import {
  Bot,
  ChevronRight,
  Compass,
  Edit3,
  Plus,
  RefreshCw,
  Search,
  Server,
  Sparkles,
  Trash2,
  Users,
  X,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { DEFAULT_AGENT_CONFIG } from '../components/AgentDefaults';
import { ToastContainer } from '../components/Toast';
import { agentService, type AgentConfig } from '../services/AgentService';

import {
  AGENT_MARKET_CATEGORIES,
  filterMarketAgents,
  type AgentMarketCategoryId,
} from './agentCatalog';
import '../home.css';
import { AgentChatView } from './AgentChatView';

type AgentCatalogScope = 'market' | 'mine';

interface AgentCatalogState {
  hasMore: boolean;
  items: AgentConfig[];
  page: number;
}

const EMPTY_CATALOG: AgentCatalogState = {
  hasMore: false,
  items: [],
  page: 1,
};

const AGENT_ACCENTS = [
  'from-cyan-400 to-blue-600',
  'from-violet-400 to-fuchsia-600',
  'from-emerald-400 to-teal-600',
  'from-amber-400 to-orange-600',
] as const;

interface AgentCreateDialogProps {
  agent?: AgentConfig;
  onClose: () => void;
  onSaved: () => void;
  open: boolean;
}

function AgentCreateDialog({ agent, onClose, onSaved, open }: AgentCreateDialogProps) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [type, setType] = useState<AgentConfig['type']>('normal');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string>();

  useEffect(() => {
    setName(agent?.name ?? '');
    setDescription(agent?.description ?? '');
    setType(agent?.type ?? 'normal');
    setError(undefined);
  }, [agent, open]);

  if (!open) {
    return null;
  }

  const saveAgent = async () => {
    if (!name.trim()) {
      return;
    }

    setCreating(true);
    setError(undefined);
    try {
      if (agent?.id) {
        await agentService.updateAgent(agent.id, {
          description: description.trim(),
          name: name.trim(),
          type,
        });
      } else {
        await agentService.createAgent({
          ...DEFAULT_AGENT_CONFIG,
          description: description.trim(),
          name: name.trim(),
          type,
        });
      }
      onSaved();
    } catch (saveError) {
      console.error('Failed to save agent.', saveError);
      setError('保存 Agent 失败，请检查服务连接后重试。');
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 p-6 backdrop-blur-sm" role="presentation" onMouseDown={onClose}>
      <div aria-labelledby="create-agent-title" aria-modal="true" className="w-full max-w-lg overflow-hidden rounded-2xl border border-white/[0.09] bg-[#1b1b1e] shadow-2xl shadow-black/50" onMouseDown={(event) => event.stopPropagation()} role="dialog">
        <div className="flex items-center justify-between border-b border-white/[0.07] px-6 py-5">
          <div>
            <h2 className="font-semibold text-zinc-100" id="create-agent-title">{agent ? '编辑 Agent' : '创建 Agent'}</h2>
            <p className="mt-1 text-xs text-zinc-500">{agent ? '更新 Agent 的基础配置。' : '建立可复用的 Agent 基础配置。'}</p>
          </div>
          <button aria-label="关闭" className="rounded-lg p-2 text-zinc-500 transition hover:bg-white/[0.06] hover:text-white" onClick={onClose} type="button">
            <X size={18} />
          </button>
        </div>

        <div className="space-y-5 px-6 py-6">
          <label className="block">
            <span className="mb-2 block text-sm font-medium text-zinc-300">名称</span>
            <input autoFocus className="w-full rounded-xl border border-white/[0.08] bg-black/20 px-4 py-3 text-sm text-zinc-100 outline-none placeholder:text-zinc-700 focus:border-cyan-500/50" onChange={(event) => setName(event.target.value)} placeholder="例如：产品研究助手" value={name} />
          </label>

          <div>
            <span className="mb-2 block text-sm font-medium text-zinc-300">运行类型</span>
            <div className="grid grid-cols-2 gap-3">
              {([
                { id: 'normal', icon: Bot, label: '普通 Agent', description: '在当前工作区中运行' },
                { id: 'independent', icon: Server, label: '独立 Agent', description: '使用独立运行环境' },
              ] as const).map(({ id, icon: Icon, label, description: typeDescription }) => (
                <button className={`rounded-xl border p-4 text-left transition ${type === id ? 'border-cyan-400/50 bg-cyan-400/[0.08]' : 'border-white/[0.07] bg-black/10 hover:border-white/[0.14]'}`} key={id} onClick={() => setType(id)} type="button">
                  <Icon className={type === id ? 'text-cyan-400' : 'text-zinc-500'} size={19} />
                  <span className="mt-3 block text-sm font-medium text-zinc-200">{label}</span>
                  <span className="mt-1 block text-xs text-zinc-600">{typeDescription}</span>
                </button>
              ))}
            </div>
          </div>

          <label className="block">
            <span className="mb-2 block text-sm font-medium text-zinc-300">简介</span>
            <textarea className="h-24 w-full resize-none rounded-xl border border-white/[0.08] bg-black/20 px-4 py-3 text-sm text-zinc-100 outline-none placeholder:text-zinc-700 focus:border-cyan-500/50" onChange={(event) => setDescription(event.target.value)} placeholder="描述这个 Agent 擅长解决的问题" value={description} />
          </label>

          {error && <p className="text-sm text-rose-300">{error}</p>}
        </div>

        <div className="flex justify-end gap-3 border-t border-white/[0.07] bg-black/10 px-6 py-4">
          <button className="rounded-xl px-4 py-2.5 text-sm text-zinc-400 transition hover:bg-white/[0.05] hover:text-white" onClick={onClose} type="button">取消</button>
          <button className="rounded-xl bg-cyan-500 px-4 py-2.5 text-sm font-semibold text-[#071014] transition hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-50" disabled={!name.trim() || creating} onClick={() => void saveAgent()} type="button">
            {creating ? '保存中…' : '保存'}
          </button>
        </div>
      </div>
    </div>
  );
}

function AgentAvatar({ agent, index }: { agent: AgentConfig; index: number }) {
  if (agent.avatar) {
    return <img alt="" className="h-full w-full object-cover" src={agent.avatar} />;
  }

  return (
    <div className={`flex h-full w-full items-center justify-center bg-gradient-to-br ${AGENT_ACCENTS[index % AGENT_ACCENTS.length]}`}>
      {agent.type === 'independent' ? <Server size={24} /> : <Bot size={24} />}
    </div>
  );
}

export function AgentsHomePage() {
  const [scope, setScope] = useState<AgentCatalogScope>('market');
  const [marketCategory, setMarketCategory] = useState<AgentMarketCategoryId>('all');
  const [catalog, setCatalog] = useState<AgentCatalogState>(EMPTY_CATALOG);
  const [query, setQuery] = useState('');
  const [debouncedQuery, setDebouncedQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string>();
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [editingAgent, setEditingAgent] = useState<AgentConfig>();
  const [chatAgent, setChatAgent] = useState<AgentConfig>();

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedQuery(query.trim()), 300);
    return () => window.clearTimeout(timer);
  }, [query]);

  const loadPage = useCallback(async (page: number, append: boolean) => {
    append ? setLoadingMore(true) : setLoading(true);
    setError(undefined);

    try {
      const result = await agentService.listAgentsPage({
        page,
        pageSize: 20,
        scope,
        ...(debouncedQuery ? { q: debouncedQuery } : {}),
      });
      setCatalog((current) => ({
        hasMore: result.pageInfo.hasMore,
        items: append ? [...current.items, ...result.items] : result.items,
        page,
      }));
    } catch (loadError) {
      console.error('Failed to load agent catalog.', loadError);
      setError('Agent 列表暂时无法加载，请检查登录状态或服务连接。');
    } finally {
      append ? setLoadingMore(false) : setLoading(false);
    }
  }, [debouncedQuery, scope]);

  useEffect(() => {
    void loadPage(1, false);
  }, [loadPage]);

  const handleDelete = async (agent: AgentConfig) => {
    if (!agent.id || !window.confirm(`确定删除 Agent「${agent.name}」吗？`)) {
      return;
    }

    try {
      await agentService.deleteAgent(agent.id);
      setCatalog((current) => ({
        ...current,
        items: current.items.filter((item) => item.id !== agent.id),
      }));
    } catch (deleteError) {
      console.error('Failed to delete agent.', deleteError);
      setError('删除 Agent 失败，请稍后重试。');
    }
  };

  const visibleAgents = scope === 'market'
    ? filterMarketAgents(catalog.items, marketCategory)
    : catalog.items;

  if (chatAgent?.id) {
    return (
      <AgentChatView
        agentId={chatAgent.id}
        agentName={chatAgent.name}
        welcomeMessage={chatAgent.welcomeMessage}
        onBack={() => setChatAgent(undefined)}
      />
    );
  }

  return (
    <div className="relative flex h-full min-h-0 flex-col overflow-hidden bg-[#111113] text-zinc-100">
      <ToastContainer />
      <div className="pointer-events-none absolute inset-x-0 top-0 h-72 bg-[radial-gradient(circle_at_20%_0%,rgba(34,211,238,0.11),transparent_56%)]" />

      <header className="relative z-10 flex shrink-0 flex-wrap items-center justify-between gap-4 border-b border-white/[0.07] px-4 py-4 sm:px-7 sm:py-5">
        <div>
          <div className="mb-1 flex items-center gap-2 text-xs font-semibold uppercase text-cyan-400">
            <Sparkles size={14} />
            Agent Studio
          </div>
          <h1 className="text-2xl font-semibold">Agent 工作台</h1>
          <p className="mt-1 text-sm text-zinc-500">发现、创建并配置你的智能 Agent。</p>
        </div>
        <button
          className="flex items-center gap-2 rounded-xl bg-cyan-500 px-4 py-2.5 text-sm font-semibold text-[#071014] shadow-lg shadow-cyan-500/15 transition hover:bg-cyan-400"
          onClick={() => setCreateModalOpen(true)}
          type="button"
        >
          <Plus size={17} />
          创建 Agent
        </button>
      </header>

      <div className="sdkwork-agents-home-layout relative z-10 flex min-h-0 flex-1 flex-col md:flex-row">
        <aside className="sdkwork-agents-home-sidebar flex w-full shrink-0 border-b border-white/[0.06] bg-black/10 p-3 md:w-56 md:flex-col md:border-b-0 md:border-r md:p-4">
          <p className="hidden px-3 pb-2 pt-1 text-[11px] font-semibold uppercase text-zinc-600 md:block">Agent 列表</p>
          <button
            className={`flex items-center gap-3 rounded-xl px-3 py-3 text-sm transition ${scope === 'market' ? 'bg-white/[0.07] text-white' : 'text-zinc-500 hover:bg-white/[0.04] hover:text-zinc-300'}`}
            onClick={() => setScope('market')}
            type="button"
          >
            <Compass className={scope === 'market' ? 'text-cyan-400' : ''} size={18} />
            发现 Agent
          </button>
          <button
            className={`ml-1 flex items-center gap-3 rounded-xl px-3 py-3 text-sm transition md:ml-0 md:mt-1 ${scope === 'mine' ? 'bg-white/[0.07] text-white' : 'text-zinc-500 hover:bg-white/[0.04] hover:text-zinc-300'}`}
            onClick={() => setScope('mine')}
            type="button"
          >
            <Bot className={scope === 'mine' ? 'text-cyan-400' : ''} size={18} />
            我的 Agent
          </button>
          <div className="mt-auto hidden rounded-xl border border-white/[0.06] bg-white/[0.025] p-3 text-xs leading-5 text-zinc-500 md:block">
            Agent 能力由 SDKWork Agents App SDK 提供，配置保存后可在各客户端复用。
          </div>
        </aside>

        <main className="min-h-0 min-w-0 flex-1 overflow-y-auto px-4 py-5 sm:px-7 sm:py-6">
          <div className="mb-6 flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <h2 className="text-lg font-semibold">{scope === 'market' ? '发现 Agent' : '我的 Agent'}</h2>
              <p className="mt-1 text-sm text-zinc-500">{scope === 'market' ? '浏览可用的 Agent 能力。' : '管理你创建的 Agent。'}</p>
            </div>
            <label className="relative block w-full max-w-sm">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-600" size={16} />
              <input
                className="w-full rounded-xl border border-white/[0.08] bg-black/20 py-2.5 pl-10 pr-4 text-sm text-zinc-200 outline-none transition placeholder:text-zinc-700 focus:border-cyan-500/50 focus:bg-black/30"
                onChange={(event) => setQuery(event.target.value)}
                placeholder="搜索 Agent"
                type="search"
                value={query}
              />
            </label>
          </div>

          {scope === 'market' && (
            <div aria-label="Agent 市场分类" className="mb-6 flex gap-2 overflow-x-auto pb-1" role="group">
              {AGENT_MARKET_CATEGORIES.map((category) => (
                <button
                  aria-pressed={marketCategory === category.id}
                  className={`shrink-0 rounded-full border px-3.5 py-1.5 text-xs font-medium transition ${marketCategory === category.id ? 'border-cyan-400/40 bg-cyan-400/[0.09] text-cyan-300' : 'border-white/[0.07] bg-white/[0.025] text-zinc-500 hover:border-white/[0.13] hover:text-zinc-300'}`}
                  key={category.id}
                  onClick={() => setMarketCategory(category.id)}
                  type="button"
                >
                  {category.label}
                </button>
              ))}
            </div>
          )}

          {error && (
            <div className="mb-5 flex flex-col items-start justify-between gap-2 rounded-xl border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-200 sm:flex-row sm:items-center">
              <span>{error}</span>
              <button className="flex shrink-0 items-center gap-1.5 whitespace-nowrap text-xs font-semibold hover:text-white" onClick={() => void loadPage(1, false)} type="button">
                <RefreshCw size={14} /> 重试
              </button>
            </div>
          )}

          {loading ? (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
              {Array.from({ length: 6 }, (_, index) => <div className="h-48 animate-pulse rounded-2xl border border-white/[0.05] bg-white/[0.025]" key={index} />)}
            </div>
          ) : visibleAgents.length === 0 ? (
            <div className="flex min-h-72 flex-col items-center justify-center rounded-2xl border border-dashed border-white/[0.09] bg-white/[0.02] text-center">
              <Bot className="mb-4 text-zinc-700" size={38} />
              <p className="font-medium text-zinc-300">没有找到 Agent</p>
              <p className="mt-1 text-sm text-zinc-600">{scope === 'mine' ? '创建第一个 Agent，开始配置专属能力。' : '尝试调整搜索关键词。'}</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
              {visibleAgents.map((agent, index) => (
                <article className="group flex min-h-52 flex-col rounded-2xl border border-white/[0.07] bg-[#19191c]/90 p-5 shadow-xl shadow-black/10 transition hover:-translate-y-0.5 hover:border-cyan-400/30 hover:bg-[#1d1d20]" key={agent.id ?? `${agent.name}-${index}`}>
                  <div className="flex items-start justify-between gap-3">
                    <div className="h-12 w-12 overflow-hidden rounded-xl text-white shadow-lg shadow-black/20">
                      <AgentAvatar agent={agent} index={index} />
                    </div>
                    {scope === 'mine' && agent.id && (
                      <div className="flex opacity-0 transition group-hover:opacity-100">
                        <button aria-label={`编辑 ${agent.name}`} className="rounded-lg p-2 text-zinc-500 hover:bg-white/[0.06] hover:text-cyan-300" onClick={() => setEditingAgent(agent)} type="button">
                          <Edit3 size={15} />
                        </button>
                        <button aria-label={`删除 ${agent.name}`} className="rounded-lg p-2 text-zinc-500 hover:bg-rose-500/10 hover:text-rose-300" onClick={() => void handleDelete(agent)} type="button">
                          <Trash2 size={15} />
                        </button>
                      </div>
                    )}
                  </div>
                  <h3 className="mt-4 truncate font-semibold text-zinc-100">{agent.name}</h3>
                  <p className="mt-2 line-clamp-2 flex-1 text-sm leading-6 text-zinc-500">{agent.description || '尚未添加描述'}</p>
                  <div className="mt-4 flex items-center justify-between border-t border-white/[0.06] pt-4 text-xs text-zinc-600">
                    <span className="flex items-center gap-1.5"><Users size={13} /> {agent.users || '0'}</span>
                    {agent.id ? (
                      <div className="flex items-center gap-3">
                        <button className="flex items-center gap-1 text-zinc-400 hover:text-cyan-300" onClick={() => setChatAgent(agent)} type="button">对话 <ChevronRight size={13} /></button>
                        {scope === 'mine' && <button className="flex items-center gap-1 text-zinc-400 hover:text-cyan-300" onClick={() => setEditingAgent(agent)} type="button">配置 <ChevronRight size={13} /></button>}
                      </div>
                    ) : (
                      <span>{agent.author || 'SDKWork'}</span>
                    )}
                  </div>
                </article>
              ))}
            </div>
          )}

          {catalog.hasMore && !loading && (
            <div className="flex justify-center py-8">
              <button className="rounded-xl border border-white/[0.08] bg-white/[0.03] px-5 py-2.5 text-sm text-zinc-400 transition hover:bg-white/[0.06] hover:text-white disabled:opacity-50" disabled={loadingMore} onClick={() => void loadPage(catalog.page + 1, true)} type="button">
                {loadingMore ? '加载中…' : '加载更多'}
              </button>
            </div>
          )}
        </main>
      </div>

      <AgentCreateDialog
        agent={editingAgent}
        open={createModalOpen || Boolean(editingAgent)}
        onClose={() => {
          setCreateModalOpen(false);
          setEditingAgent(undefined);
        }}
        onSaved={() => {
          setCreateModalOpen(false);
          setEditingAgent(undefined);
          setScope('mine');
          void loadPage(1, false);
        }}
      />
    </div>
  );
}
