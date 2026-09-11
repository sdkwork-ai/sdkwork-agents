import type { ChatMessage } from '../types';

export interface ChatTurn {
  id: string;
  user?: ChatMessage;
  assistant?: ChatMessage;
}

/** Groups a flat transcript into user/assistant turn pairs for industry-style layout. */
export function groupMessagesIntoTurns(messages: ChatMessage[]): ChatTurn[] {
  const turns: ChatTurn[] = [];
  let current: ChatTurn | null = null;

  for (const message of messages) {
    if (message.role === 'user') {
      if (current) {
        turns.push(current);
      }
      current = { id: `turn-${message.id}`, user: message };
      continue;
    }

    if (message.role === 'model') {
      if (!current) {
        current = { id: `turn-${message.id}`, assistant: message };
      } else {
        current.assistant = message;
      }
    }
  }

  if (current) {
    turns.push(current);
  }

  return turns;
}
