import type { AgentWebhookEventType } from './agent-webhook-event-type';

export interface AgentWebhookSubscription {
  webhookId: string;
  url: string;
  eventTypes: AgentWebhookEventType[];
  status: 'active' | 'disabled';
  description?: string;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}
