import {
  Accessibility,
  AudioLines,
  Image as ImageIcon,
  Music,
  PlaySquare,
  Smile,
  Wand2,
  Waves,
} from "lucide-react";
import type { ComponentType } from "react";

/**
 * The single creation-type taxonomy shared by the inspiration feature cards,
 * the unified creative input box and the creative page dispatcher.
 *
 * One list, one place: the cards and the input box used to carry two separate
 * enums with different members, so a capability could be selectable from one
 * surface and impossible to reach from the other — and sound effects had no
 * entry anywhere. Add a capability here once and every surface picks it up.
 */
export type CreativeCreationTypeId =
  | "agent"
  | "image"
  | "video"
  | "music"
  | "voice"
  | "digital_human"
  | "action"
  | "sfx";

/** Creation types that map onto a generations app API command. */
export type CreativeGenerationMode = Exclude<CreativeCreationTypeId, "agent">;

export interface CreativeCreationTypeDefinition {
  id: CreativeCreationTypeId;
  label: string;
  icon: ComponentType<{ size?: number | string; className?: string }>;
  /** Inspiration-card presentation: emoji badge plus tailwind gradient. */
  card: {
    emoji: string;
    description: string;
    background: string;
    tag?: string;
  };
}

export const CREATIVE_CREATION_TYPES: readonly CreativeCreationTypeDefinition[] = [
  {
    id: "image",
    label: "图片生成",
    icon: ImageIcon,
    card: {
      emoji: "🖼️",
      description: "智能美学提升",
      background: "bg-gradient-to-br from-blue-500 to-indigo-500",
      tag: "New",
    },
  },
  {
    id: "video",
    label: "视频生成",
    icon: PlaySquare,
    card: {
      emoji: "🎬",
      description: "Seedance 2.0",
      background: "bg-gradient-to-br from-purple-500 to-violet-500",
    },
  },
  {
    id: "music",
    label: "音乐生成",
    icon: Music,
    card: {
      emoji: "🎵",
      description: "文生音乐 / 歌词成曲",
      background: "bg-gradient-to-br from-pink-500 to-rose-500",
    },
  },
  {
    id: "voice",
    label: "配音生成",
    icon: AudioLines,
    card: {
      emoji: "🎙️",
      description: "文本转语音",
      background: "bg-gradient-to-br from-cyan-500 to-sky-500",
    },
  },
  {
    id: "sfx",
    label: "音效生成",
    icon: Waves,
    card: {
      emoji: "🔊",
      description: "描述即得音效",
      background: "bg-gradient-to-br from-amber-400 to-orange-500",
    },
  },
  {
    id: "digital_human",
    label: "数字人",
    icon: Smile,
    card: {
      emoji: "🙂",
      description: "形象 + 口型驱动",
      background: "bg-gradient-to-br from-emerald-400 to-teal-500",
    },
  },
  {
    id: "action",
    label: "动作模仿",
    icon: Accessibility,
    card: {
      emoji: "🤸",
      description: "参考动作迁移",
      background: "bg-gradient-to-br from-fuchsia-500 to-purple-500",
    },
  },
  {
    id: "agent",
    label: "Agent 模式",
    icon: Wand2,
    card: {
      emoji: "🤖",
      description: "多步任务编排",
      background: "bg-gradient-to-br from-emerald-400 to-teal-400",
    },
  },
];

/**
 * Modality carried over the wire. `sound_effects` is the generations app API's
 * own path segment for the sound-effect command (`/generations/sound_effects`),
 * not the `sfx` label the UI uses.
 */
export type CreativeGenerationModality =
  | "image"
  | "video"
  | "music"
  | "voice"
  | "sound_effects"
  | "digital_human"
  | "action";

/**
 * Every operation the generations app API exposes. The generated app SDK has a
 * typed method for all of them except `avatar` and `motion_mimicry`, which are
 * registered by the router and declared in the app OpenAPI contract but are not
 * yet part of the checked-in generated client.
 */
export type CreativeGenerationOperationType =
  | "text_to_image"
  | "image_edit"
  | "text_to_video"
  | "image_to_video"
  | "video_extend"
  | "text_to_music"
  | "lyrics_to_music"
  | "sound_effects"
  | "speech"
  | "transcription"
  | "translation"
  | "avatar"
  | "motion_mimicry";

export interface CreativeGenerationDispatch {
  modality: CreativeGenerationModality;
  operationType: CreativeGenerationOperationType;
}

/**
 * The operation a modality runs when the caller does not name one. This is the
 * only place that pairing is written down; the command layer reuses it so a
 * bare modality can never fall back to `text_to_image` the way the previous
 * `mode === "video" ? ... : "image"` collapse did.
 */
export const CREATIVE_DEFAULT_OPERATION_BY_MODALITY: Readonly<
  Record<CreativeGenerationModality, CreativeGenerationOperationType>
> = Object.freeze({
  image: "text_to_image",
  video: "text_to_video",
  music: "text_to_music",
  voice: "speech",
  sound_effects: "sound_effects",
  digital_human: "avatar",
  action: "motion_mimicry",
});

function dispatchFor(modality: CreativeGenerationModality): CreativeGenerationDispatch {
  return { modality, operationType: CREATIVE_DEFAULT_OPERATION_BY_MODALITY[modality] };
}

const CREATIVE_GENERATION_DISPATCH: Record<CreativeGenerationMode, CreativeGenerationDispatch> = {
  image: dispatchFor("image"),
  video: dispatchFor("video"),
  music: dispatchFor("music"),
  voice: dispatchFor("voice"),
  sfx: dispatchFor("sound_effects"),
  digital_human: dispatchFor("digital_human"),
  action: dispatchFor("action"),
};

const CREATIVE_GENERATION_MODES = new Set<string>(
  Object.keys(CREATIVE_GENERATION_DISPATCH),
);

export function isCreativeCreationTypeId(value: string | undefined | null): value is CreativeCreationTypeId {
  return CREATIVE_CREATION_TYPES.some((type) => type.id === value);
}

export function isCreativeGenerationMode(value: string | undefined | null): value is CreativeGenerationMode {
  return typeof value === "string" && CREATIVE_GENERATION_MODES.has(value);
}

/** Resolve a definition for display; unknown ids fall back to the first entry. */
export function resolveCreativeCreationType(
  id: string | undefined | null,
): CreativeCreationTypeDefinition {
  return CREATIVE_CREATION_TYPES.find((type) => type.id === id) ?? CREATIVE_CREATION_TYPES[0];
}

/**
 * Resolve the generations command for a creation mode.
 *
 * Returns `null` for a non-generation mode (`agent`) or an unknown id, so the
 * caller can keep agent prompts out of the generation pipeline instead of
 * silently downgrading them. `hasReferenceImages` upgrades the image and video
 * defaults to their reference-driven siblings rather than collapsing either
 * one onto text-to-image.
 */
export function resolveCreativeGenerationDispatch(
  mode: string | undefined | null,
  options: { hasReferenceImages?: boolean } = {},
): CreativeGenerationDispatch | null {
  if (!isCreativeGenerationMode(mode)) {
    return null;
  }
  const dispatch = CREATIVE_GENERATION_DISPATCH[mode];
  if (!options.hasReferenceImages) {
    return dispatch;
  }
  if (dispatch.modality === "image") {
    return { modality: "image", operationType: "image_edit" };
  }
  if (dispatch.modality === "video") {
    return { modality: "video", operationType: "image_to_video" };
  }
  return dispatch;
}
