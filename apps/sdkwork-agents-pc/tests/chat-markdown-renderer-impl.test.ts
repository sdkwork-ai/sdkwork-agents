import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const require = createRequire(fileURLToPath(new URL('../package.json', import.meta.url)));
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');
const ReactMarkdown = require('react-markdown').default;
const remarkGfm = require('remark-gfm').default;
const rehypeSanitize = require('rehype-sanitize').default;

import { getChatCodeTokenClassName } from '../packages/sdkwork-agents-pc-commons/src/components/ChatCodeBlock';
import { chatMarkdownSanitizeSchema } from '../packages/sdkwork-agents-pc-commons/src/components/chatMarkdownSanitizeSchema';
import { prepareChatMarkdownSource } from '../packages/sdkwork-agents-pc-commons/src/components/chatMarkdownUtils';

function renderAssistantMessage(content: string): string {
  const markdown = prepareChatMarkdownSource(content, false);
  return renderToStaticMarkup(
    React.createElement(ReactMarkdown, {
      remarkPlugins: [remarkGfm],
      rehypePlugins: [[rehypeSanitize, chatMarkdownSanitizeSchema]],
      children: markdown,
    }),
  );
}

test('chat code token classifier highlights numbers and keywords', () => {
  assert.equal(getChatCodeTokenClassName('42', 'javascript'), 'chat-md-code-token-number');
  assert.equal(getChatCodeTokenClassName('const', 'javascript'), 'chat-md-code-token-keyword');
});

test('MarkdownRendererImpl renders bold and inline code', () => {
  const html = renderAssistantMessage('Use **bold** and `inline` here.');
  assert.match(html, /<strong/);
  assert.match(html, /<code/);
  assert.match(html, /inline/);
});

test('MarkdownRendererImpl renders fenced code without leaking raw fence markers', () => {
  const html = renderAssistantMessage('```javascript\nconst answer = 42;\n```');
  assert.match(html, /<code/);
  assert.match(html, /const answer = 42/);
  assert.doesNotMatch(html, /```javascript/);
});
