import type { AgentWebhookEventType } from './agent-webhook-event-type';

export interface CreateWebhookSubscriptionRequest {
  webhookId: string;
  url: string;
  eventTypes: AgentWebhookEventType[];
  description?: string;
}
