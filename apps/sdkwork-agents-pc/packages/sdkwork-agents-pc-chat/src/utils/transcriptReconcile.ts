import type { ChatMessage } from '../types';

/**
 * Reconcile optimistic client transcript rows with the server-authoritative
 * transcript after a turn completes. Matches turns by index and copies stable
 * server item ids onto local rows without clobbering in-flight optimistic tails.
 */
export function reconcileTranscriptWithServer(
  localMessages: ChatMessage[],
  serverMessages: ChatMessage[],
): ChatMessage[] {
  if (serverMessages.length === 0) {
    return localMessages;
  }

  const mergedPrefix = localMessages.slice(0, serverMessages.length).map((message, index) => {
    const serverMessage = serverMessages[index];
    if (!serverMessage) {
      return message;
    }
    if (message.role !== serverMessage.role) {
      return message;
    }
    return {
      ...message,
      id: serverMessage.id,
      feedback: serverMessage.feedback ?? message.feedback,
      feedbackVersion: serverMessage.feedbackVersion ?? message.feedbackVersion,
      mediaResources: serverMessage.mediaResources ?? message.mediaResources,
      images: serverMessage.images ?? message.images,
    };
  });

  if (localMessages.length <= serverMessages.length) {
    return mergedPrefix;
  }

  return [...mergedPrefix, ...localMessages.slice(serverMessages.length)];
}
