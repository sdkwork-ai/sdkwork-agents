import { defaultSchema } from 'rehype-sanitize';

/** GFM task lists and chat-safe links for assistant markdown. */
export const chatMarkdownSanitizeSchema = {
  ...defaultSchema,
  tagNames: [...(defaultSchema.tagNames || []), 'input', 'section'],
  attributes: {
    ...defaultSchema.attributes,
    a: [...(defaultSchema.attributes?.a || []), ['target'], ['rel']],
    code: [...(defaultSchema.attributes?.code || []), ['className']],
    input: [
      ...(defaultSchema.attributes?.input || []),
      ['checked'],
      ['disabled'],
      ['type'],
    ],
    section: [...(defaultSchema.attributes?.section || []), ['className'], ['dataFootnotes']],
    li: [...(defaultSchema.attributes?.li || []), ['className']],
    th: [...(defaultSchema.attributes?.th || []), ['align']],
    td: [...(defaultSchema.attributes?.td || []), ['align']],
  },
  protocols: {
    ...defaultSchema.protocols,
    href: ['http', 'https', 'mailto'],
    src: ['http', 'https'],
  },
};
