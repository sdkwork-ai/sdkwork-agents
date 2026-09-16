import React from 'react';
import { cn } from '@sdkwork/agents-pc-commons';
import { CREATIVE_CREATION_TYPES } from '@sdkwork/agents-pc-core/sdk/creationTypes';

interface FeatureCardProps {
  id: string;
  title: string;
  desc: string;
  tag?: string;
  icon: string;
  bg: string;
}

/**
 * Workspace entries. They are not creation types: selecting one switches to
 * another tab instead of changing the input box mode.
 */
const WORKSPACE_CARDS: FeatureCardProps[] = [
  { id: 'octo', title: 'Magic Studio', desc: 'Vibe create, 创作自然流动', tag: 'Beta', icon: '✨', bg: 'bg-gradient-to-br from-orange-400 to-rose-400' },
  { id: 'canvas', title: '无限画布', desc: '自由创作', icon: '🎨', bg: 'bg-gradient-to-br from-blue-400 to-cyan-400' },
];

/**
 * Generation entries are derived from the shared creation-type taxonomy rather
 * than hard-coded here. The list used to be five fixed entries, which left
 * music / voice / sound-effect / digital-human / action unreachable from the
 * inspiration page even though the input box could select them.
 */
const GENERATION_CARDS: FeatureCardProps[] = CREATIVE_CREATION_TYPES.map((type) => ({
  id: type.id,
  title: type.label,
  desc: type.card.description,
  icon: type.card.emoji,
  bg: type.card.background,
  ...(type.card.tag ? { tag: type.card.tag } : {}),
}));

const FEATURE_CARDS: FeatureCardProps[] = [...WORKSPACE_CARDS, ...GENERATION_CARDS];

interface InspirationFeatureCardsProps {
  inputBoxMode: string;
  setInputBoxMode: (mode: string) => void;
}

export const InspirationFeatureCards: React.FC<InspirationFeatureCardsProps> = ({ inputBoxMode, setInputBoxMode }) => {
  return (
    <div className="w-full grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3 mb-16">
      {FEATURE_CARDS.map(card => (
        <button 
          key={card.id} 
          onClick={() => {
            if (card.id === 'canvas') {
              window.dispatchEvent(new CustomEvent('switch-tab', { detail: { tab: 'canvas' } }));
            } else {
              setInputBoxMode(card.id);
            }
          }}
          className={cn(
            "bg-[#1e1e1e] hover:bg-[#252525] border rounded-xl p-3 flex items-center gap-3 transition-colors text-left relative group",
            inputBoxMode === card.id ? "border-cyan-500/50 bg-[#252525]" : "border-white/5"
          )}
        >
          <div className={cn("w-10 h-10 rounded-lg flex items-center justify-center text-lg shrink-0", card.bg)}>
            {card.icon}
          </div>
          <div className="flex flex-col overflow-hidden">
            <span className="text-[13px] font-medium text-zinc-200 truncate">{card.title}</span>
            <span className="text-[11px] text-zinc-500 truncate">{card.desc}</span>
          </div>
          {card.tag && (
            <div className="absolute -top-2 -right-2 bg-cyan-500 text-black text-[9px] font-bold px-1.5 py-0.5 rounded shadow-sm">
              {card.tag}
            </div>
          )}
        </button>
      ))}
    </div>
  );
};
