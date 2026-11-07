import type { MediaToolDirectoryEntry } from './media-tool-directory-entry';

/** Tool directory entry result following SdkWorkApiResponse envelope. */
export interface ToolDirectoryEntryApiResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 鎼?5.3. */
  code: 0;
  data: unknown & MediaToolDirectoryEntry;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
