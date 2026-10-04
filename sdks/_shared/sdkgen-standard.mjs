import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const SDKWORK_SDKGEN_STANDARD = Object.freeze({
  standardProfile: 'sdkwork-v3',
  typescriptCommonPackage: Object.freeze({
    name: '@sdkwork/sdk-common',
    version: '^1.0.5',
  }),
  canonicalRootWin: String.raw`..\sdkwork-sdk-generator`,
  canonicalEntrypointWin: String.raw`..\sdkwork-sdk-generator\bin\sdkgen.js`,
  canonicalEntrypointPosix: '../sdkwork-sdk-generator/bin/sdkgen.js',
  envOverride: 'SDKWORK_SDKGEN_PATH',
  deprecatedEntrypointFragment: ['java', 'source'].join(''),
  generatedOutput: 'generated/server-openapi',
});

const defaultSdkgenPath = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../../../sdkwork-sdk-generator/bin/sdkgen.js',
);

export function resolveSdkgenEntrypoint(env = process.env) {
  const override = env[SDKWORK_SDKGEN_STANDARD.envOverride];
  if (override) {
    return path.resolve(override);
  }
  if (fs.existsSync(defaultSdkgenPath)) {
    return defaultSdkgenPath;
  }
  return SDKWORK_SDKGEN_STANDARD.canonicalEntrypointPosix;
}
