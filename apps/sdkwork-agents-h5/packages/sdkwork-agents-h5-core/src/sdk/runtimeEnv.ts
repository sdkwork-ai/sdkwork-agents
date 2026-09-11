/**
 * Reads Vite (`import.meta.env`) and Node (`process.env`) configuration in one place.
 * Contract tests run under tsx without a Vite env shim.
 */
export function readRuntimeEnv(key: string): string | undefined {
  const meta = (import.meta as unknown as { env?: Record<string, unknown> }).env;
  const fromMeta = meta?.[key];
  if (typeof fromMeta === "string") {
    const trimmed = fromMeta.trim();
    if (trimmed.length > 0) return trimmed;
  }
  const fromProcess = (globalThis as unknown as { process?: { env?: Record<string, unknown> } }).process
    ?.env?.[key];
  if (typeof fromProcess === "string") {
    const trimmed = fromProcess.trim();
    if (trimmed.length > 0) return trimmed;
  }
  return undefined;
}
