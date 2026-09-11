import React, { useState, useEffect } from 'react';
import { ChevronDown } from 'lucide-react';
import { CreativeInputBox } from '@sdkwork/agents-pc-commons';
import { ImageDetailModal } from './components/ImageDetailModal';
import { VideoDetailModal } from './components/VideoDetailModal';
import { ActivityDetailView } from './components/ActivityDetailView';
import { DiscoverTab } from './components/tabs/DiscoverTab';
import { SkillsTab } from './components/tabs/SkillsTab';
import { ShortVideosTab } from './components/tabs/ShortVideosTab';
import { ActivitiesTab } from './components/tabs/ActivitiesTab';
import { InspirationFeatureCards } from './components/InspirationFeatureCards';
import { InspirationTabs } from './components/InspirationTabs';
import { InspirationService } from './services/InspirationService';
import type {
  Activity,
  ActivityWork,
  DiscoverData,
  ShortVideo,
  SkillCategory,
} from './types';

export const InspirationView = () => {
  const [activeTab, setActiveTab] = useState<string>('发现');
  const [inputBoxMode, setInputBoxMode] = useState<string>('agent');

  const [discoverData, setDiscoverData] = useState<DiscoverData | null>(null);
  const [filteredVideos, setFilteredVideos] = useState<ShortVideo[]>([]);
  const [filteredSkills, setFilteredSkills] = useState<SkillCategory[]>([]);
  const [filteredActivities, setFilteredActivities] = useState<Activity[]>([]);

  useEffect(() => {
    // Initial data load
    InspirationService.getDiscoverData()
      .then(setDiscoverData)
      .catch(() => setDiscoverData(null));
  }, []);

  const [searchQuery, setSearchQuery] = useState<string>('');
  
  // Real-time Search Updates via Service
  useEffect(() => {
    InspirationService.getShortVideos(searchQuery).then(setFilteredVideos);
    InspirationService.getSkills(searchQuery).then(setFilteredSkills);
    InspirationService.getActivities(searchQuery).then(setFilteredActivities);
  }, [searchQuery]);

  const handleInputSubmit = (value: string, mode: string) => {
    sessionStorage.setItem('pending_creative_prompt', value);
    sessionStorage.setItem('pending_creative_mode', mode);
    window.dispatchEvent(new CustomEvent('switch-tab', { detail: { tab: 'creative' } }));
  };
  const [selectedImage, setSelectedImage] = useState<any>(null);
  const [selectedVideo, setSelectedVideo] = useState<ShortVideo | ActivityWork | null>(null);
  const [activeActivity, setActiveActivity] = useState<Activity | null>(null);

  // If a specific activity detail page is active, show the details instead
  if (activeActivity) {
    return (
      <div className="flex-1 w-full h-full bg-[#141414] overflow-y-auto overflow-x-hidden text-white relative">
        <ActivityDetailView 
          activity={activeActivity}
          onClose={() => setActiveActivity(null)}
          onPlayVideo={(work) => setSelectedVideo(work)}
        />
        
        <VideoDetailModal 
          isOpen={!!selectedVideo}
          onClose={() => setSelectedVideo(null)}
          video={selectedVideo}
        />
      </div>
    );
  }

  return (
    <div className="flex-1 w-full h-full bg-[#141414] overflow-y-auto overflow-x-hidden text-white relative">
      <div className="w-full py-16 flex flex-col items-center">
        {/* Top Centered Section */}
        <div className="w-full max-w-[1200px] px-8 flex flex-col items-center">
          <h1 className="text-2xl font-medium mb-12 flex items-center gap-2">
            开启你的 <span className="text-cyan-400 flex items-center cursor-pointer hover:text-cyan-300 transition-colors">图片生成 <ChevronDown size={18} className="ml-0.5" /></span> 即刻造梦！
          </h1>
          
          {/* Input Box */}
          <CreativeInputBox 
            key={inputBoxMode}
            initialMode={inputBoxMode}
            onSubmit={handleInputSubmit}
            className="w-full mb-8" 
          />

          {/* Feature Cards */}
          <InspirationFeatureCards 
            inputBoxMode={inputBoxMode} 
            setInputBoxMode={setInputBoxMode} 
          />
        </div>

        {/* Gallery Section - Full Width */}
        <div className="w-full px-4 md:px-8 lg:px-12 xl:px-16">
          <InspirationTabs 
            activeTab={activeTab} 
            setActiveTab={setActiveTab}
            searchQuery={searchQuery}
            setSearchQuery={setSearchQuery}
          />

          {/* Tab Views */}
          
          {/* 1. DISCOVER TAB VIEW */}
          {activeTab === '发现' && discoverData && (
            <DiscoverTab 
              discoverData={discoverData} 
              onSelectImage={setSelectedImage} 
            />
          )}

          {/* 2. SKILL TAB VIEW */}
          {activeTab === '技能' && (
            <SkillsTab 
              filteredSkills={filteredSkills} 
            />
          )}

          {/* 3. SHORT VIDEO TAB VIEW */}
          {activeTab === '短片' && (
            <ShortVideosTab 
              filteredVideos={filteredVideos} 
              onPlayVideo={setSelectedVideo} 
            />
          )}

          {/* 4. ACTIVITY TAB VIEW */}
          {activeTab === '活动' && (
            <ActivitiesTab 
              filteredActivities={filteredActivities} 
              onSelectActivity={setActiveActivity} 
            />
          )}

        </div>
      </div>

      {/* Media Detail Modals */}
      <ImageDetailModal 
        isOpen={!!selectedImage} 
        onClose={() => setSelectedImage(null)} 
        image={selectedImage} 
      />

      <VideoDetailModal 
        isOpen={!!selectedVideo}
        onClose={() => setSelectedVideo(null)}
        video={selectedVideo}
      />
    </div>
  );
};
