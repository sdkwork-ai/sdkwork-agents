import type {
  CreativeModelDefinition,
  CreativeModelModality,
} from './creativeModelCatalogTypes';

/**
 * Static baseline catalog.
 *
 * Seeded from the `sdkwork-models` repository static catalog (`models/` +
 * `catalogVersion`) and materialized at build time. This is the default data
 * source for every creation mode: it works offline and before the first
 * remote sync. The remote sync layer may add, update, deprecate or remove
 * models on top of this baseline (see creativeModelCatalogService).
 */

export const STATIC_CREATIVE_MODEL_CATALOG_VERSION = '2026.09.06.1';

function staticModel(
  modality: CreativeModelModality,
  order: number,
  input: Omit<CreativeModelDefinition, 'modality' | 'lifecycle' | 'source' | 'order'> &
    Partial<Pick<CreativeModelDefinition, 'lifecycle'>>,
): CreativeModelDefinition {
  return {
    lifecycle: 'active',
    ...input,
    modality,
    source: 'static',
    order,
  };
}

export const STATIC_IMAGE_MODELS: CreativeModelDefinition[] = [
  staticModel('image', 0, { id: '5.0-lite', label: '图片 5.0 Lite', desc: '指令响应更精准，生成效果更智能' }),
  staticModel('image', 1, { id: '4.7', label: '图片 4.7', desc: '画质全面优化，指令响应能力再次提升' }),
  staticModel('image', 2, { id: '4.6', label: '图片 4.6', desc: '人像一致性保持更好，性价比更高' }),
  staticModel('image', 3, { id: '4.5', label: '图片 4.5', desc: '强化一致性、风格与图文响应' }),
  staticModel('image', 4, { id: '4.1', label: '图片 4.1', desc: '更专业的创意、美学和一致性保持' }),
];

export const STATIC_VIDEO_MODELS: CreativeModelDefinition[] = [
  staticModel('video', 0, { id: '2.0-mini', label: '即梦 Seedance 2.0 mini', desc: '极具性价比，相近的体验，比Fast更快的推理速度', spark: true }),
  staticModel('video', 1, { id: '2.0-fast-vip', label: '即梦 Seedance 2.0 Fast VIP', desc: '极速推理，会员专属通道，音视文图均可参考（暂不支持真人人脸）', spark: true }),
  staticModel('video', 2, { id: '2.0-vip', label: '即梦 Seedance 2.0 VIP', desc: '全模态能力，会员专属通道，音视文图均可参考（暂不支持真人人脸）', spark: true }),
  staticModel('video', 3, { id: '2.0-fast', label: '即梦 Seedance 2.0 Fast', desc: '高性价比，音视文图均可参考（暂不支持真人人脸）' }),
  staticModel('video', 4, { id: '2.0', label: '即梦 Seedance 2.0', desc: '全能王者，音视文图均可参考（暂不支持真人人脸）' }),
];

export const STATIC_MUSIC_MODELS: CreativeModelDefinition[] = [
  staticModel('music', 0, { id: 'music_pro', label: '即梦音乐 Pro', desc: '根据文本提示词和参考音频生成高质量音乐', spark: true }),
  staticModel('music', 1, { id: 'music_1.0', label: '即梦音乐 1.0', desc: '强大的音乐生成能力，支持多种曲风' }),
  staticModel('music', 2, { id: 'suno_v3.5', label: 'Suno v3.5', desc: '生成流派融合的音乐及人声演唱' }),
  staticModel('music', 3, { id: 'udio', label: 'Udio', desc: '惊艳的高保真人声与复杂的音乐结构' }),
  staticModel('music', 4, { id: 'stable_audio', label: 'Stable Audio', desc: '高质量的纯音乐与环境音效生成' }),
];

export const STATIC_VOICE_MODELS: CreativeModelDefinition[] = [
  staticModel('voice', 0, { id: 'voice_pro', label: '即梦配音 Pro', desc: '根据文本生成高质量语音', spark: true }),
  staticModel('voice', 1, { id: 'voice_1.0', label: '即梦配音 1.0', desc: '支持多种音色与情绪控制' }),
];

export const STATIC_DIGITAL_HUMAN_MODELS: CreativeModelDefinition[] = [
  staticModel('digital_human', 0, {
    id: 'master_mode',
    label: '大师模式',
    desc: '电影级的表演效果',
    spark: true,
    image: 'https://images.unsplash.com/photo-1544005313-94ddf0286df2?auto=format&fit=crop&w=120&h=120&q=80',
  }),
  staticModel('digital_human', 1, {
    id: 'fast_mode',
    label: '快速模式',
    desc: '更低成本，快速生成',
    image: 'https://images.unsplash.com/photo-1517841905240-472988babdf9?auto=format&fit=crop&w=120&h=120&q=80',
  }),
  staticModel('digital_human', 2, {
    id: 'basic_mode',
    label: '基础模式',
    desc: '仅仅修改人物口型。适合演讲、对白',
    image: 'https://images.unsplash.com/photo-1539571696357-5a69c17a67c6?auto=format&fit=crop&w=120&h=120&q=80',
  }),
];

export const STATIC_ACTION_MODELS: CreativeModelDefinition[] = [
  staticModel('action', 0, { id: 'master', label: '大师', desc: '效果最佳，画质超清', spark: true, badge: '1.5', subBadge: 'PRO', isNew: true }),
  staticModel('action', 1, { id: 'vivid', label: '生动', desc: '不限画幅，动效更真', badge: '2.0', isNew: true }),
  staticModel('action', 2, { id: 'fast', label: '快速', desc: '更快生成，成本更低', badge: '2.0', isNew: true }),
];

export const STATIC_CREATIVE_MODELS: Readonly<
  Record<CreativeModelModality, readonly CreativeModelDefinition[]>
> = Object.freeze({
  image: STATIC_IMAGE_MODELS,
  video: STATIC_VIDEO_MODELS,
  music: STATIC_MUSIC_MODELS,
  voice: STATIC_VOICE_MODELS,
  digital_human: STATIC_DIGITAL_HUMAN_MODELS,
  action: STATIC_ACTION_MODELS,
});

/** Default selection per modality (first entry of each static catalog). */
export const STATIC_DEFAULT_MODEL_IDS: Readonly<
  Record<CreativeModelModality, string>
> = Object.freeze({
  image: '5.0-lite',
  video: '2.0-mini',
  music: 'music_pro',
  voice: 'voice_pro',
  digital_human: 'fast_mode',
  action: 'vivid',
});

/**
 * Remote catalog query mapping per creative modality, expressed with the
 * `modalities` filter of `GET /app/v3/api/ai/models`. Modalities mapped to an
 * empty array are static-only (the models app API has no matching catalog
 * facet yet) and skip remote sync.
 */
export const REMOTE_MODALITY_QUERY: Readonly<
  Record<CreativeModelModality, readonly string[]>
> = Object.freeze({
  image: ['image'],
  video: ['video'],
  music: ['music'],
  voice: ['audio'],
  digital_human: [],
  action: [],
});

/** Creative modalities the remote sync covers, in deterministic order. */
export const REMOTE_SYNC_MODALITIES: readonly CreativeModelModality[] = (
  Object.keys(REMOTE_MODALITY_QUERY) as CreativeModelModality[]
).filter((modality) => REMOTE_MODALITY_QUERY[modality].length > 0);
