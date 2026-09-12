/**
 * Media extraction and progress labeling for chat tool calls.
 *
 * Two tool families carry media results in the turn loop:
 * - `mcp__generations__*` (built-in generations MCP): result payload is
 *   `{generation: {modality, ...}, results: [{resourceSnapshot: {kind, url}}], mediaUrls: [...]}`.
 * - media family tools (`image.*`, `video.*`, `audio.*`, `music.*`): result
 *   payload is `{taskId, status, items: [{kind, url}]}`.
 *
 * The parsers are pure so the tool-card state machine (placeholder → result)
 * is testable without React.
 */

export type ToolMediaKind = 'image' | 'video' | 'audio';

export interface ToolMedia {
  kind: ToolMediaKind;
  url: string;
}

/** Maps a tool id to the media kind its results render as (null = not media). */
export function toolMediaKind(toolId: string): ToolMediaKind | null {
  const name = toolId.toLowerCase();
  if (name.includes('image')) return 'image';
  if (name.includes('video')) return 'video';
  if (/(audio|speech|voice|music|sound)/.test(name)) return 'audio';
  return null;
}

function normalizeKind(raw: unknown, fallback: ToolMediaKind): ToolMediaKind {
  const value = typeof raw === 'string' ? raw.toLowerCase() : '';
  if (value === 'image') return 'image';
  if (value === 'video') return 'video';
  if (/(audio|voice|speech|music|sound|sfx)/.test(value)) return 'audio';
  return fallback;
}

function dedupeUrls(items: ToolMedia[]): ToolMedia[] {
  const seen = new Set<string>();
  return items.filter((item) => {
    if (!item.url || seen.has(item.url)) return false;
    seen.add(item.url);
    return true;
  });
}

/**
 * Extracts renderable media from one tool-result payload (raw JSON string or
 * already-parsed object). Returns an empty list for non-media tools, empty
 * payloads, or unparsable JSON — the caller renders the JSON summary instead.
 */
export function extractToolMedia(toolId: string, result: unknown): ToolMedia[] {
  const fallbackKind = toolMediaKind(toolId);
  if (fallbackKind == null) return [];
  let payload: unknown = result;
  if (typeof result === 'string') {
    try {
      payload = JSON.parse(result);
    } catch {
      return [];
    }
  }
  if (payload == null || typeof payload !== 'object') return [];
  const root = payload as Record<string, unknown>;

  const items: ToolMedia[] = [];

  // Media family shape: `{items: [{kind, url}]}`.
  if (Array.isArray(root.items)) {
    for (const item of root.items) {
      if (item == null || typeof item !== 'object') continue;
      const record = item as Record<string, unknown>;
      const url = typeof record.url === 'string' ? record.url.trim() : '';
      if (!url) continue;
      items.push({ kind: normalizeKind(record.kind, fallbackKind), url });
    }
  }

  // Generations shape: `{results: [{resourceSnapshot: {kind, url}}]}`.
  if (Array.isArray(root.results)) {
    for (const result_ of root.results) {
      if (result_ == null || typeof result_ !== 'object') continue;
      const record = result_ as Record<string, unknown>;
      const snapshot = record.resourceSnapshot;
      const url =
        snapshot != null && typeof snapshot === 'object'
          ? (snapshot as Record<string, unknown>).url
          : undefined;
      if (typeof url !== 'string' || !url.trim()) continue;
      const resultType = typeof record.resultType === 'string' ? record.resultType : undefined;
      items.push({
        kind: normalizeKind(resultType ?? (snapshot as Record<string, unknown>).kind, fallbackKind),
        url: url.trim(),
      });
    }
  }

  // Generations convenience list: `{mediaUrls: [...]}` (kind from the tool).
  if (Array.isArray(root.mediaUrls) && items.length === 0) {
    for (const url of root.mediaUrls) {
      if (typeof url !== 'string' || !url.trim()) continue;
      items.push({ kind: fallbackKind, url: url.trim() });
    }
  }

  return dedupeUrls(items);
}

/** i18n key of the in-progress placeholder label for one tool. */
export function toolProgressKey(toolId: string): string | null {
  const kind = toolMediaKind(toolId);
  switch (kind) {
    case 'image':
      return 'toolProgress.image';
    case 'video':
      return 'toolProgress.video';
    case 'audio':
      return 'toolProgress.audio';
    default:
      return null;
  }
}

/** Fallback (zh-CN) placeholder label when the i18n bundle lacks the key. */
export const TOOL_PROGRESS_DEFAULTS: Record<string, string> = {
  'toolProgress.image': '图片生成中…',
  'toolProgress.video': '视频生成中…',
  'toolProgress.audio': '音频合成中…',
};

/** Fallback (zh-CN) completed label prefix, e.g. `toolDone.image` → 图片已生成. */
export const TOOL_DONE_DEFAULTS: Record<string, string> = {
  'toolDone.image': '图片已生成',
  'toolDone.video': '视频已生成',
  'toolDone.audio': '音频已生成',
};
