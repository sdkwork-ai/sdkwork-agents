import { useCallback, useState } from 'react';

/**
 * Wire protocols accepted by the cloudrouter gateway turn pipeline. Ids must
 * match the backend `wireProtocol` enum on `CreateAgentTurnRequest`.
 */
export const WIRE_PROTOCOL_OPTIONS = [
  {
    id: 'chat_completions',
    labelKey: 'wireProtocolChatCompletions',
    descriptionKey: 'wireProtocolChatCompletionsDesc',
  },
  {
    id: 'anthropic_messages',
    labelKey: 'wireProtocolAnthropic',
    descriptionKey: 'wireProtocolAnthropicDesc',
  },
  {
    id: 'google_content',
    labelKey: 'wireProtocolGoogle',
    descriptionKey: 'wireProtocolGoogleDesc',
  },
  {
    id: 'openai_responses',
    labelKey: 'wireProtocolResponses',
    descriptionKey: 'wireProtocolResponsesDesc',
  },
] as const;

export type WireProtocolId = (typeof WIRE_PROTOCOL_OPTIONS)[number]['id'];

const WIRE_PROTOCOL_STORAGE_KEY = 'sdkwork-agents-chat-wire-protocol';
const DEFAULT_WIRE_PROTOCOL: WireProtocolId = 'chat_completions';

/** Reads the persisted playground wire protocol, falling back to the default. */
export function readStoredWireProtocol(): WireProtocolId {
  try {
    const stored = localStorage.getItem(WIRE_PROTOCOL_STORAGE_KEY);
    if (stored && WIRE_PROTOCOL_OPTIONS.some((option) => option.id === stored)) {
      return stored as WireProtocolId;
    }
  } catch {
    // ignore storage failures (private mode, quota, etc.)
  }
  return DEFAULT_WIRE_PROTOCOL;
}

/** Playground-wide LLM wire protocol selection, persisted in localStorage. */
export const useWireProtocol = () => {
  const [wireProtocol, setWireProtocolState] = useState<WireProtocolId>(readStoredWireProtocol);

  const setWireProtocol = useCallback((next: WireProtocolId) => {
    setWireProtocolState(next);
    try {
      localStorage.setItem(WIRE_PROTOCOL_STORAGE_KEY, next);
    } catch {
      // ignore storage failures
    }
  }, []);

  return { wireProtocol, setWireProtocol };
};
