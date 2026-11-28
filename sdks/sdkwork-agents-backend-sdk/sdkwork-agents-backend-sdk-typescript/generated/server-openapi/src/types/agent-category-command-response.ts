/** Command acceptance for an agent category mutation following the SdkWorkApiResponse envelope. */
export interface AgentCategoryCommandResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & { accepted: boolean; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
