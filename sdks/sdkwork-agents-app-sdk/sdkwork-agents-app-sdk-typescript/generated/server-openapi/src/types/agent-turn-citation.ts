export interface AgentTurnCitation {
  /** 1-based marker number used in the grounding prompt. */
  index: number;
  documentId: string;
  title: string;
  sourceUri?: string | null;
  score?: number | null;
}
