import assert from 'node:assert/strict';
import { existsSync, lstatSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const pcRoot = path.join(repoRoot, 'apps/sdkwork-agents-pc');

function read(relativePath) {
  return readFileSync(path.join(pcRoot, relativePath), 'utf8');
}

function collectSourceFiles(relativeRoot) {
  const pending = [path.join(pcRoot, relativeRoot)];
  const files = [];
  while (pending.length > 0) {
    const current = pending.pop();
    const stat = lstatSync(current);
    if (stat.isSymbolicLink()) {
      continue;
    }
    if (stat.isDirectory()) {
      for (const entry of readdirSync(current)) {
        if (entry === 'node_modules' || entry === 'dist') {
          continue;
        }
        pending.push(path.join(current, entry));
      }
    } else if (/\.(?:ts|tsx|js|jsx|mjs|cjs)$/u.test(current)) {
      files.push(current);
    }
  }
  return files;
}

test('PC uploads use the composed Drive Uploader with canonical media identity', () => {
  const service = read('packages/sdkwork-agents-pc-core/src/sdk/driveUploadService.ts');
  assert.match(service, /client\(\)\.uploader\.uploadByProfile|this\.getClient\(\)\.uploader\.uploadByProfile/u);
  assert.match(service, /drive:\/\/spaces\/\$\{uploadItem\.spaceId\}\/nodes\/\$\{uploadItem\.nodeId\}/u);
  assert.match(service, /requestedTtlSeconds:\s*900/u);
  assert.match(service, /retention:\s*\{\s*mode:\s*["']long_term["']/u);
  assert.doesNotMatch(service, /\bfetch\s*\(|\baxios\b|\/upload_sessions|\/drive\/uploader/u);
});

test('PC production paths do not restore local upload or fake chat backends', () => {
  const packageJson = read('package.json');
  assert.equal(existsSync(path.join(pcRoot, 'server.ts')), false);
  assert.doesNotMatch(packageJson, /"express"|"@google\/genai"|"dotenv"/u);

  for (const file of collectSourceFiles('packages')) {
    const source = readFileSync(file, 'utf8');
    assert.doesNotMatch(
      source,
      /\/api\/chat|GoogleGenAI|GEMINI_API_KEY|mock_presentation|mockUrl|Mocking PPTX|\/app\/v3\/api\/drive\/(?:uploader|upload_sessions)/u,
      path.relative(pcRoot, file),
    );
  }
});

test('PC production composition exposes every SDK-backed workbench package', () => {
  const workbench = read('src/components/WorkbenchLayout.tsx');
  const sidebar = read('src/components/GlobalSidebar.tsx');
  const tabs = read('src/components/workbenchTabs.ts');
  const workspace = read('src/agents/AgentWorkspace.tsx');
  const agentsHomeEntry = read('packages/sdkwork-agents-pc-agents/src/home.ts');
  const agentsHomePage = read('packages/sdkwork-agents-pc-agents/src/pages/AgentsHomePage.tsx');
  // The home conversation is rendered inline through the shared chat package,
  // whose ChatService owns the composed Drive upload path.
  const agentsChatView = read('packages/sdkwork-agents-pc-agents/src/pages/AgentChatView.tsx');
  const homeRuntime = read('packages/sdkwork-agents-pc-agents/src/services/AgentsHomeRuntime.ts');

  for (const packageName of [
    '@sdkwork/agents-pc-assets',
    '@sdkwork/agents-pc-canvas',
    '@sdkwork/agents-pc-chat',
    '@sdkwork/agents-pc-creative',
    '@sdkwork/agents-pc-inspiration',
    '@sdkwork/agents-pc-presentation',
  ]) {
    assert.match(workbench, new RegExp(packageName.replace('/', '\\/'), 'u'), packageName);
  }
  for (const tab of ['agents', 'chat_session', 'inspiration', 'creative', 'assets', 'presentation', 'canvas']) {
    assert.match(tabs, new RegExp(`['\"]${tab}['\"]`, 'u'), tab);
    assert.match(sidebar, new RegExp(`${tab}:|id:\\s*['\"]${tab}['\"]`, 'u'), tab);
  }
  assert.doesNotMatch(workbench, /@\/packages\//u);
  assert.match(workspace, /AgentsHomePage/u);
  assert.match(workspace, /@sdkwork\/agents-pc-agents\/home/u);
  assert.match(agentsHomeEntry, /AgentsHomeRuntime/u);
  assert.match(agentsHomePage, /AgentChatView/u);
  assert.match(agentsHomePage, /\.\.\/services\/AgentService/u);
  assert.match(agentsChatView, /@sdkwork\/agents-pc-chat\/ChatView/u);
  assert.match(homeRuntime, /@sdkwork\/agents-pc-core\/sdk\/agentsAppSdkClient/u);
  assert.match(homeRuntime, /@sdkwork\/agents-pc-core\/sdk\/driveAppSdkClient/u);
});

test('Blob and data URLs remain limited to explicit local-only rendering or export files', () => {
  const allowedLocalOnlyFiles = [
    'packages/sdkwork-agents-pc-agents/src/services/DefaultAvatarService.ts',
    'packages/sdkwork-agents-pc-canvas/src/components/CanvasSnapshotPanel.tsx',
    'packages/sdkwork-agents-pc-canvas/src/hooks/useCanvasLogic.ts',
    'packages/sdkwork-agents-pc-chat/src/components/ArtifactPanel.tsx',
    'packages/sdkwork-agents-pc-presentation/src/PPTView.tsx',
  ];
  const uploadFiles = [
    'packages/sdkwork-agents-pc-agents/src/components/EditBasicInfoModal.tsx',
    'packages/sdkwork-agents-pc-agents/src/components/MessageInput.tsx',
    'packages/sdkwork-agents-pc-agents/src/pages/AgentChatView.tsx',
    'packages/sdkwork-agents-pc-chat/src/ChatView.tsx',
    'packages/sdkwork-agents-pc-commons/src/components/CreativeInputBox.tsx',
  ];

  for (const file of allowedLocalOnlyFiles) {
    assert.match(read(file), /URL\.createObjectURL|data:(?:image|text)/u, `${file} must remain an explicit local-only exception`);
  }
  for (const file of uploadFiles) {
    assert.doesNotMatch(read(file), /readAsDataURL|URL\.createObjectURL|blob:|data:(?:image|audio|video|application)/u, file);
  }
});
