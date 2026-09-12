import type { AgentWebhookEventType } from './agent-webhook-event-type';

export interface AgentWebhookSubscriptionCreated {
  webhookId: string;
  url: string;
  eventTypes: AgentWebhookEventType[];
  status: 'active' | 'disabled';
  /** One-time echo of the HMAC signing secret. */
  secret: string;
  description?: string;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}
