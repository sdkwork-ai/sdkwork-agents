export interface AgentCallOutputSpec {
  format?: 'json' | 'xml' | 'text';
  schema?: Record<string, unknown>;
  rootElement?: string;
  strict?: boolean;
}
