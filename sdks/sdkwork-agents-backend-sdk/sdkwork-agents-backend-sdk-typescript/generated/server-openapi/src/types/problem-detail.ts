import type { FieldError } from './field-error';

export interface ProblemDetail {
  type: string;
  title: string;
  status: number;
  detail?: string;
  instance?: string;
  /** Numeric error result code. MUST be non-zero. See API_SPEC.md 鎼?5.3. */
  code: number;
  /** Server-owned request correlation id. Same semantics as SdkWorkApiResponse.traceId. */
  traceId: string;
  errors?: FieldError[];
}
