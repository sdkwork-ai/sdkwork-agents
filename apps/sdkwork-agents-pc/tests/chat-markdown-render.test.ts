import assert from 'node:assert/strict';
import test from 'node:test';
import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeSanitize from 'rehype-sanitize';

import { chatMarkdownSanitizeSchema } from '../packages/sdkwork-agents-pc-commons/src/components/chatMarkdownSanitizeSchema';
import { prepareChatMarkdownSource } from '../packages/sdkwork-agents-pc-commons/src/components/chatMarkdownUtils';

function renderMarkdown(content: string, streaming = false): string {
  const markdown = prepareChatMarkdownSource(content, streaming);
  return renderToStaticMarkup(
    React.createElement(ReactMarkdown, {
      remarkPlugins: [remarkGfm],
      rehypePlugins: [[rehypeSanitize, chatMarkdownSanitizeSchema]],
      children: markdown,
    }),
  );
}

test('react-markdown renders fenced code blocks', () => {
  const html = renderMarkdown('Here is code:\n\n```javascript\nconst x = 1;\n```\n');
  assert.match(html, /<code/);
  assert.match(html, /const x = 1/);
});

test('react-markdown renders headings and lists', () => {
  const html = renderMarkdown('## Title\n\n- one\n- two\n');
  assert.match(html, /<h2[^>]*>Title<\/h2>/);
  assert.match(html, /<ul/);
  assert.match(html, /<li>one<\/li>/);
});
