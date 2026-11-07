export interface SdkWorkApiResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  /** Operation-specific payload. Typed per operation through allOf or explicit schema refs. */
  data: unknown;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
