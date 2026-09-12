import type { AgentWebhookEventType } from './agent-webhook-event-type';

export interface AgentWebhookDelivery {
  deliveryId: string;
  webhookId: string;
  eventType: AgentWebhookEventType;
  payload: Record<string, unknown>;
  signature: string;
  status: 'queued' | 'succeeded' | 'failed';
  responseCode?: number;
  errorDetail?: string;
  createdAt: string;
  completedAt?: string;
}
