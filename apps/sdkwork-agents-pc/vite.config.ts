import path from 'node:path';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig, type Plugin } from 'vite';

function serializeCredentialEntryBootstrapForInlineScript(token: string): string {
  return JSON.stringify(token)
    .replaceAll('<', '\\u003c')
    .replaceAll('>', '\\u003e')
    .replaceAll('&', '\\u0026');
}

function createAgentsCredentialEntryBootstrapPlugin(
  mode: string,
  accessToken: string,
): Plugin | undefined {
  if (mode !== 'development' || !accessToken) {
    return undefined;
  }

  return {
    name: 'sdkwork-agents-iam-credential-entry-bootstrap',
    apply: 'serve',
    transformIndexHtml: {
      order: 'pre',
      handler: (html) => ({
        html,
        tags: [
          {
            tag: 'script',
            children:
              'globalThis.__SDKWORK_CREDENTIAL_ENTRY_BOOTSTRAP_ACCESS_TOKEN__ = '
              + `${serializeCredentialEntryBootstrapForInlineScript(accessToken)};`,
            injectTo: 'head-prepend',
          },
        ],
      }),
    },
  };
}

export default defineConfig(({ mode }) => {
  const credentialEntryBootstrapAccessToken = process.env.SDKWORK_ACCESS_TOKEN ?? '';

  return {
    plugins: [
      createAgentsCredentialEntryBootstrapPlugin(mode, credentialEntryBootstrapAccessToken),
      react(),
      tailwindcss(),
    ],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, '.'),
      },
      dedupe: ['react', 'react-dom'],
    },
    build: {
      rollupOptions: {
        output: {
          manualChunks(id) {
            const normalizedId = id.replaceAll('\\\\', '/');
            if (normalizedId.includes('/sdks/sdkwork-agents-app-sdk/')) return 'sdk-agents';
            if (normalizedId.includes('/sdkwork-iam/sdks/sdkwork-iam-app-sdk/')) return 'sdk-iam';
            if (normalizedId.includes('/sdkwork-drive/sdks/sdkwork-drive-app-sdk/')) return 'sdk-drive';
            if (normalizedId.includes('/sdkwork-assets/sdks/sdkwork-assets-app-sdk/')) return 'sdk-assets';
            if (normalizedId.includes('/sdkwork-community/sdks/sdkwork-community-app-sdk/')) return 'sdk-community';
            if (normalizedId.includes('/sdkwork-generations/sdks/sdkwork-generations-app-sdk/')) return 'sdk-generations';
            if (normalizedId.includes('/sdkwork-knowledgebase/')) return 'sdk-knowledgebase';
            if (normalizedId.includes('/sdkwork-skills/')) return 'sdk-skills';
            if (normalizedId.includes('/sdkwork-voice/')) return 'sdk-voice';
            if (!id.includes('node_modules')) return undefined;
            return undefined;
          },
        },
      },
    },
    server: {
      host: '0.0.0.0',
      hmr: process.env.DISABLE_HMR !== 'true',
      port: 5195,
      proxy: {
        '/app/v3/api': 'http://127.0.0.1:8095',
        '/healthz': 'http://127.0.0.1:8095',
        '/livez': 'http://127.0.0.1:8095',
        '/readyz': 'http://127.0.0.1:8095',
      },
      watch: process.env.DISABLE_HMR === 'true' ? null : {},
    },
  };
});
