/**
 * Chat-oriented markdown normalization for streaming assistant output.
 * Closes unclosed fences during SSE deltas so partial code blocks render
 * stably instead of flickering into raw text.
 */

function readMarkdownFenceMarker(line: string): string | null {
  const match = /^(?: {0,3})(`{3,}|~{3,})/.exec(line);
  return match?.[1] ?? null;
}

export function readUnclosedMarkdownFence(content: string): {
  activeFence: string | null;
  fenceCount: number;
} {
  let activeFence: string | null = null;
  let fenceCount = 0;
  const parts = content.split(/(\r\n|\n|\r)/);

  for (let index = 0; index < parts.length; index += 2) {
    const marker = readMarkdownFenceMarker(parts[index] || '');
    if (!marker) {
      continue;
    }
    if (!activeFence) {
      activeFence = marker;
      fenceCount += 1;
      continue;
    }
    if (marker[0] === activeFence[0] && marker.length >= activeFence.length) {
      activeFence = null;
      fenceCount += 1;
    }
  }

  return { activeFence, fenceCount };
}

/** Appends a closing fence while the model is still streaming inside a block. */
export function normalizeStreamingMarkdown(content: string): string {
  const { activeFence } = readUnclosedMarkdownFence(content);
  if (!activeFence) {
    return content;
  }
  return `${content}\n${activeFence}`;
}

function mapMarkdownLinesOutsideCodeFences(
  content: string,
  transform: (line: string) => string,
): string {
  let activeFence: string | null = null;
  const parts = content.split(/(\r\n|\n|\r)/);
  let result = '';

  for (let index = 0; index < parts.length; index += 2) {
    const line = parts[index] || '';
    const newline = parts[index + 1] || '';
    const marker = readMarkdownFenceMarker(line);
    if (marker && !activeFence) {
      activeFence = marker;
      result += line + newline;
      continue;
    }
    if (marker && activeFence && marker[0] === activeFence[0] && marker.length >= activeFence.length) {
      activeFence = null;
      result += line + newline;
      continue;
    }
    result += (activeFence ? line : transform(line)) + newline;
  }

  return result;
}

function isMarkdownAutolinkToken(token: string): boolean {
  return /^<https?:\/\//i.test(token)
    || /^<mailto:/i.test(token)
    || /^<[^\s@<>]+@[^\s@<>]+\.[^\s@<>]+>$/i.test(token);
}

function escapeHtmlToken(token: string): string {
  return token
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

function escapeRawHtmlTags(line: string): string {
  return line.replace(
    /<!--[\s\S]*?-->|<\/?[A-Za-z][A-Za-z0-9:-]*(?:\s+[^<>]*)?\/?>|<![A-Za-z][^<>]*>/g,
    (token) => (isMarkdownAutolinkToken(token) ? token : escapeHtmlToken(token)),
  );
}

function decodeTransportEscapedMarkdown(value: string): string {
  if (!/(?:\\r\\n|\\n|\\r)/.test(value)) {
    return value;
  }
  return value
    .replace(/\\r\\n/g, '\n')
    .replace(/\\n/g, '\n')
    .replace(/\\r/g, '\n')
    .replace(/\\t/g, '\t');
}

function trimTrailingHorizontalWhitespace(value: string): string {
  return value.replace(/[ \t]+$/g, '');
}

function isMarkdownStructuralLine(trimmedLine: string): boolean {
  return /^(?:#{1,6}\s|[-*+]\s|\d{1,9}\.\s|>\s?|`{3,}|~{3,}|\[\^[^\]]+\]:)/.test(trimmedLine)
    || /^\|.*\|$/.test(trimmedLine)
    || /^:?-{3,}:?(?:\s*\|\s*:?-{3,}:?)+$/.test(trimmedLine)
    || /^(?:[-*_]\s*){3,}$/.test(trimmedLine);
}

function shouldAppendSoftLineBreak(line: string, nextLine: string): boolean {
  const trimmed = line.trim();
  const nextTrimmed = nextLine.trim();
  if (!trimmed || !nextTrimmed || /(?: {2}|\\)$/.test(line)) {
    return false;
  }
  return !isMarkdownStructuralLine(trimmed) && !isMarkdownStructuralLine(nextTrimmed);
}

/** Preserves single newlines inside prose while keeping block markdown intact. */
export function normalizeSoftLineBreaks(value: string): string {
  let activeFence: string | null = null;
  const parts = value.split(/(\r\n|\n|\r)/);
  let result = '';

  for (let index = 0; index < parts.length; index += 2) {
    const line = parts[index] || '';
    const newline = parts[index + 1] || '';
    const marker = readMarkdownFenceMarker(line);
    if (marker && !activeFence) {
      activeFence = marker;
      result += line + newline;
      continue;
    }
    if (marker && activeFence && marker[0] === activeFence[0] && marker.length >= activeFence.length) {
      activeFence = null;
      result += line + newline;
      continue;
    }
    if (activeFence) {
      result += line + newline;
      continue;
    }

    const nextLine = parts[index + 2] || '';
    result += shouldAppendSoftLineBreak(line, nextLine)
      ? `${trimTrailingHorizontalWhitespace(line)}  ${newline}`
      : line + newline;
  }

  return result;
}

/** Light cleanup for model output before markdown parse. */
export function normalizeChatMarkdownContent(content: string): string {
  if (!content) {
    return '';
  }
  const extracted = extractChatMessageText(content) ?? content;
  const decoded = decodeTransportEscapedMarkdown(extracted);
  const normalized = decoded
    .replace(/\r\n/g, '\n')
    .replace(/\u0000/g, '')
    .replace(/^\s*data:\s*/gm, '')
    .trimEnd();
  return normalizeSoftLineBreaks(
    mapMarkdownLinesOutsideCodeFences(normalized, escapeRawHtmlTags),
  );
}

export function prepareChatMarkdownSource(
  content: string,
  streaming: boolean,
): string {
  const normalized = normalizeChatMarkdownContent(content);
  return streaming ? normalizeStreamingMarkdown(normalized) : normalized;
}

export function isSafeMarkdownHref(href: string): boolean {
  const normalized = href.trim().toLowerCase();
  return (
    normalized.startsWith('http://')
    || normalized.startsWith('https://')
    || normalized.startsWith('mailto:')
    || /^#[a-z0-9][a-z0-9._~:-]*$/i.test(href.trim())
  );
}

function parseJsonLikeMarkdownPayload(value: string): unknown | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const first = trimmed[0];
  const last = trimmed[trimmed.length - 1];
  const mayBeJson = (first === '{' && last === '}')
    || (first === '[' && last === ']')
    || (first === '"' && last === '"');
  if (!mayBeJson) {
    return null;
  }
  try {
    return JSON.parse(trimmed);
  } catch {
    return null;
  }
}

function readMarkdownTextFromUnknown(value: unknown, depth: number): string | null {
  if (depth > 5 || value === null || value === undefined) {
    return null;
  }
  if (typeof value === 'string') {
    const parsed = parseJsonLikeMarkdownPayload(value);
    if (parsed !== null) {
      const nested = readMarkdownTextFromUnknown(parsed, depth + 1);
      if (nested !== null) {
        return nested;
      }
    }
    return value;
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value);
  }
  if (Array.isArray(value)) {
    const parts = value
      .map((item) => readMarkdownTextFromUnknown(item, depth + 1))
      .filter((item): item is string => Boolean(item?.trim()));
    return parts.length > 0 ? parts.join('\n') : null;
  }
  if (typeof value !== 'object') {
    return null;
  }
  const record = value as Record<string, unknown>;
  for (const key of ['outputText', 'output_text', 'textDelta', 'text_delta', 'delta', 'text', 'content', 'message']) {
    if (!Object.prototype.hasOwnProperty.call(record, key)) {
      continue;
    }
    const text = readMarkdownTextFromUnknown(record[key], depth + 1);
    if (text?.trim()) {
      return text;
    }
  }
  return null;
}

/** Unwrap JSON transport envelopes that some gateways stream as a single string. */
export function extractChatMessageText(content: string): string | null {
  const trimmed = content.trim();
  if (!trimmed) {
    return null;
  }
  return readMarkdownTextFromUnknown(trimmed, 0);
}
