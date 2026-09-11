import assert from 'node:assert/strict';
import test from 'node:test';

import {
  isSafeMarkdownHref,
  normalizeChatMarkdownContent,
  normalizeStreamingMarkdown,
  prepareChatMarkdownSource,
  readUnclosedMarkdownFence,
} from '../packages/sdkwork-agents-pc-commons/src/components/chatMarkdownUtils';

test('readUnclosedMarkdownFence detects open code fence', () => {
  const content = 'intro\n```ts\nconst x = 1;\n';
  assert.deepEqual(readUnclosedMarkdownFence(content), {
    activeFence: '```',
    fenceCount: 1,
  });
});

test('normalizeStreamingMarkdown closes open fence', () => {
  const content = '```python\nprint("hi")';
  assert.equal(
    normalizeStreamingMarkdown(content),
    '```python\nprint("hi")\n```',
  );
});

test('normalizeChatMarkdownContent escapes raw html outside fences', () => {
  const content = 'Hello <script>alert(1)</script>\n```html\n<div>ok</div>\n```';
  const normalized = normalizeChatMarkdownContent(content);
  assert.match(normalized, /&lt;script&gt;/);
  assert.match(normalized, /<div>ok<\/div>/);
});

test('prepareChatMarkdownSource applies streaming fence repair', () => {
  const content = '```js\nconsole.log(1)';
  assert.equal(
    prepareChatMarkdownSource(content, true),
    '```js\nconsole.log(1)\n```',
  );
  assert.equal(
    prepareChatMarkdownSource(content, false),
    '```js\nconsole.log(1)',
  );
});

test('isSafeMarkdownHref allows http and fragment links', () => {
  assert.equal(isSafeMarkdownHref('https://example.com'), true);
  assert.equal(isSafeMarkdownHref('#section-1'), true);
  assert.equal(isSafeMarkdownHref('javascript:alert(1)'), false);
});
