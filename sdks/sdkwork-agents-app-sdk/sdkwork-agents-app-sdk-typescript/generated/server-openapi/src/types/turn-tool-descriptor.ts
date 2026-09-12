/** One tool the model may call during a turn. */
export interface TurnToolDescriptor {
  toolId: string;
  name: string;
  description: string;
  inputSchema?: Record<string, unknown>;
  requiresApproval?: boolean;
  policyCategory?: string | null;
  timeoutMs: string;
  origin: 'builtinGenerations' | 'builtinMedia' | 'externalMcp';
}
