const DEFAULT_SESSION_TITLE_MAX_LENGTH = 120;

/** Bounds auto-generated session titles before they are sent to the API. */
export function trimSessionTitle(text: string, maxLength = DEFAULT_SESSION_TITLE_MAX_LENGTH): string {
  const trimmed = text
    .replace(/[\r\n\t]+/g, ' ')
    .replace(/[\u0000-\u001F\u007F]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
  if (trimmed.length <= maxLength) {
    return trimmed;
  }
  if (maxLength <= 3) {
    return trimmed.slice(0, maxLength);
  }
  return `${trimmed.slice(0, maxLength - 3)}...`;
}
