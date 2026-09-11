/**
 * Runtime environment accessor, unified on @sdkwork/sdk-common so every
 * module shares one implementation (Vite import.meta.env + Node process.env).
 */
export { readRuntimeEnv } from "@sdkwork/sdk-common";
