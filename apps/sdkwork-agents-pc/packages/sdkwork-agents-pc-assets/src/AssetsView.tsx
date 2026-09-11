import React, { useState, useEffect } from 'react';
import { HelpCircle, Inbox } from 'lucide-react';
import { AssetDetailModal, AssetItem } from './components/AssetDetailModal';
import { AssetsHeader } from './components/AssetsHeader';
import { AssetsFilter } from './components/AssetsFilter';
import { AssetsGrid } from './components/AssetsGrid';
import { AssetsService } from './services/AssetsService';

export const AssetsView = () => {
  const [activeTab, setActiveTab] = useState<'history' | 'subject' | 'canvas'>('history');
  const [activeFilter, setActiveFilter] = useState<'image' | 'video' | 'audio' | 'document'>('image');
  
  const [groups, setGroups] = useState<{ date: string; items: AssetItem[] }[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    setLoadError(null);
    AssetsService.getAssetGroups(activeFilter)
      .then((nextGroups) => {
        if (cancelled) return;
        setGroups(nextGroups);
      })
      .catch((error) => {
        if (cancelled) return;
        console.error('Failed to load asset groups', error);
        setGroups([]);
        setLoadError('资产加载失败，请确认资产服务可用后重试');
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [activeFilter]);

  // Preview states
  const [selectedItem, setSelectedItem] = useState<AssetItem | null>(null);
  const [isDetailOpen, setIsDetailOpen] = useState<boolean>(false);

  // Flattened items for seamless navigation
  const allItems = groups.flatMap(g => g.items);
  const currentIndex = selectedItem ? allItems.findIndex(i => i.id === selectedItem.id) : -1;
  const hasPrev = currentIndex > 0;
  const hasNext = currentIndex < allItems.length - 1;

  const handlePrev = () => {
    if (hasPrev) {
      setSelectedItem(allItems[currentIndex - 1]);
    }
  };

  const handleNext = () => {
    if (hasNext) {
      setSelectedItem(allItems[currentIndex + 1]);
    }
  };

  const handleItemClick = (item: AssetItem) => {
    setSelectedItem(item);
    setIsDetailOpen(true);
  };

  return (
    <div className="flex flex-col h-full w-full bg-[#f5f5f7] text-zinc-900 dark:bg-[#18181A] dark:text-gray-200">
      <AssetsHeader activeTab={activeTab} setActiveTab={setActiveTab} />
      <AssetsFilter activeFilter={activeFilter} setActiveFilter={setActiveFilter} />
      
      {loadError && (
        <div className="mx-6 mb-4 px-4 py-3 rounded-lg bg-red-500/10 border border-red-500/20 text-sm text-red-700 dark:text-red-300">
          {loadError}
        </div>
      )}

      {isLoading ? (
        <div className="flex-1 flex items-center justify-center text-sm text-zinc-500">
          加载中...
        </div>
      ) : !loadError && allItems.length === 0 ? (
        <div className="flex-1 flex flex-col items-center justify-center gap-1.5 px-6 text-center">
          <div className="w-14 h-14 rounded-full bg-zinc-200/70 dark:bg-zinc-800/70 flex items-center justify-center text-zinc-400 dark:text-zinc-500">
            <Inbox size={26} />
          </div>
          <p className="mt-2 text-sm font-semibold text-zinc-800 dark:text-white">暂无资产</p>
          <p className="text-xs text-zinc-500 dark:text-zinc-400">生成的图片、视频、音频与文档会按时间显示在这里</p>
        </div>
      ) : (
        <AssetsGrid
          groups={groups}
          activeFilter={activeFilter}
          onItemClick={handleItemClick}
        />
      )}

      {/* Floating Help Button */}
      <div className="absolute right-6 bottom-6">
        <button className="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center text-zinc-400 hover:text-white transition-colors backdrop-blur-sm">
          <HelpCircle size={16} />
        </button>
      </div>

      {/* High-Fidelity Asset Detail Modal */}
      <AssetDetailModal
        isOpen={isDetailOpen}
        onClose={() => setIsDetailOpen(false)}
        currentItem={selectedItem}
        onPrev={handlePrev}
        onNext={handleNext}
        hasPrev={hasPrev}
        hasNext={hasNext}
      />
    </div>
  );
};
