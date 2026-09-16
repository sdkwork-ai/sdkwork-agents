export interface CreativeMessage {
  id: string;
  role: 'user' | 'assistant';
  text?: string;
  stage?: 'thinking' | 'loading' | 'completed';
  progress?: number;
  imageUrl?: string;
  imageUrls?: string[];
  videoUrl?: string;
  videoUrls?: string[];
  /** Music / voice / sound-effect results (audio media). */
  audioUrl?: string;
  audioUrls?: string[];
  suggestions?: string[];
  mode?: string;
  modelInfo?: string;
}

export interface CreativeSession {
  id: string;
  title: string;
  avatarUrl?: string;
  messages: CreativeMessage[];
  isPinned?: boolean;
}
