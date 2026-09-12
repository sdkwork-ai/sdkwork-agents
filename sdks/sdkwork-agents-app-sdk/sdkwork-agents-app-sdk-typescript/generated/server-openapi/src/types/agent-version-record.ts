export interface AgentVersionRecord {
  versionId: string;
  agentId: string;
  versionNumber: string;
  manifest: Record<string, unknown>;
  defaultCodeTaskIntent?: Record<string, unknown>;
  implementationProviderId?: string;
  implementationKind?: string;
  implementationType: string;
  description?: string;
  createdBy: string;
  createdAt: string;
  activatedAt?: string;
}
