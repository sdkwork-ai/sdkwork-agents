/**
 * Handoff channel between the inspiration page (`sdkwork-agents-pc-inspiration`)
 * and the creative page (`sdkwork-agents-pc-creative`).
 *
 * The two pages live in different packages, so the prompt, the creation mode
 * and the input-box settings travel through `sessionStorage`. Both sides used to
 * repeat the key literals independently and the settings payload was dropped
 * altogether, which silently lost ratio / duration / reference images between
 * the two pages. The keys live here so the two sides cannot drift apart.
 */

const CREATIVE_HANDOFF_PROMPT_KEY = "pending_creative_prompt";
const CREATIVE_HANDOFF_MODE_KEY = "pending_creative_mode";
const CREATIVE_HANDOFF_SETTINGS_KEY = "pending_creative_settings";

export interface CreativeHandoffPayload {
  prompt: string;
  mode: string;
  settings?: unknown;
}

/** Stage a submission from the inspiration page for the creative page. */
export function writeCreativeHandoff(payload: CreativeHandoffPayload): void {
  try {
    sessionStorage.setItem(CREATIVE_HANDOFF_PROMPT_KEY, payload.prompt);
    sessionStorage.setItem(CREATIVE_HANDOFF_MODE_KEY, payload.mode);
    if (payload.settings === undefined) {
      sessionStorage.removeItem(CREATIVE_HANDOFF_SETTINGS_KEY);
    } else {
      sessionStorage.setItem(CREATIVE_HANDOFF_SETTINGS_KEY, JSON.stringify(payload.settings));
    }
  } catch (error) {
    // Private-mode / quota failures must not block the tab switch; the creative
    // page then starts from the prompt and mode alone.
    console.error("Failed to stage the creative handoff.", error);
  }
}

/**
 * Read and clear a staged submission. Returns `null` when there is nothing to
 * consume, so the caller never re-triggers a stale generation.
 */
export function consumeCreativeHandoff(): CreativeHandoffPayload | null {
  try {
    const prompt = sessionStorage.getItem(CREATIVE_HANDOFF_PROMPT_KEY);
    if (!prompt) {
      return null;
    }
    const mode = sessionStorage.getItem(CREATIVE_HANDOFF_MODE_KEY) ?? "agent";
    const rawSettings = sessionStorage.getItem(CREATIVE_HANDOFF_SETTINGS_KEY);
    sessionStorage.removeItem(CREATIVE_HANDOFF_PROMPT_KEY);
    sessionStorage.removeItem(CREATIVE_HANDOFF_MODE_KEY);
    sessionStorage.removeItem(CREATIVE_HANDOFF_SETTINGS_KEY);
    return {
      prompt,
      mode,
      settings: rawSettings ? (JSON.parse(rawSettings) as unknown) : undefined,
    };
  } catch (error) {
    console.error("Failed to read the creative handoff.", error);
    return null;
  }
}
