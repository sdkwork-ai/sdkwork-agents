import type { MediaToolDirectoryEntry } from './media-tool-directory-entry';

/** Media tool directory following SdkWorkApiResponse envelope; the directory is returned as a bare array under data. */
export interface MediaToolDirectoryResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & MediaToolDirectoryEntry[];
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
