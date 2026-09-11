import assert from 'node:assert/strict';
import test from 'node:test';

import {
  CHAT_DEFAULT_MODEL_ID,
  CHAT_SELECTED_MODEL_STORAGE_KEY,
  chatModelPickerGroups,
  persistChatSelectedModelId,
  readStoredChatSelectedModelId,
  resolveChatDefaultModelId,
  resolveChatSelectedModelId,
} from '../packages/sdkwork-agents-pc-chat/src/modelPicker/chatModelPickerCatalog';

async function withMockLocalStorage(fn: () => Promise<void> | void): Promise<void> {
  const originalDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
  const values = new Map<string, string>();
  const storage = {
    get length() {
      return values.size;
    },
    clear() {
      values.clear();
    },
    getItem(key: string) {
      return values.has(key) ? values.get(key)! : null;
    },
    key(index: number) {
      return [...values.keys()][index] ?? null;
    },
    removeItem(key: string) {
      values.delete(key);
    },
    setItem(key: string, value: string) {
      values.set(key, String(value));
    },
  } as Storage;

  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true,
    enumerable: true,
    value: storage,
  });

  try {
    await fn();
  } finally {
    if (originalDescriptor) {
      Object.defineProperty(globalThis, 'localStorage', originalDescriptor);
    } else {
      delete (globalThis as { localStorage?: Storage }).localStorage;
    }
  }
}

function firstCatalogModelId(): string {
  const modelId = chatModelPickerGroups[0]?.llms[0]?.id;
  assert.ok(modelId, 'chat model catalog must expose at least one model');
  return modelId;
}

function alternateCatalogModelId(preferred: string): string {
  for (const group of chatModelPickerGroups) {
    for (const model of group.llms) {
      if (model.id !== preferred) {
        return model.id;
      }
    }
  }
  return preferred;
}

test('resolveChatSelectedModelId falls back to default when storage is empty', async () => {
  await withMockLocalStorage(() => {
    assert.equal(readStoredChatSelectedModelId(), null);
    assert.equal(resolveChatSelectedModelId(), resolveChatDefaultModelId());
  });
});

test('persist and resolve restore the last chat header model for echo on revisit', async () => {
  await withMockLocalStorage(() => {
    const remembered = alternateCatalogModelId(CHAT_DEFAULT_MODEL_ID);
    persistChatSelectedModelId(remembered);

    assert.equal(localStorage.getItem(CHAT_SELECTED_MODEL_STORAGE_KEY), remembered);
    assert.equal(readStoredChatSelectedModelId(), remembered);
    assert.equal(resolveChatSelectedModelId(), remembered);
  });
});

test('resolveChatSelectedModelId ignores stale model ids that left the catalog', async () => {
  await withMockLocalStorage(() => {
    persistChatSelectedModelId('stale-model-that-is-not-in-catalog');
    assert.equal(resolveChatSelectedModelId(), resolveChatDefaultModelId());
  });
});

test('resolveChatSelectedModelId accepts a known model from the active picker groups', async () => {
  await withMockLocalStorage(() => {
    const known = firstCatalogModelId();
    persistChatSelectedModelId(known);
    assert.equal(resolveChatSelectedModelId(chatModelPickerGroups), known);
  });
});
