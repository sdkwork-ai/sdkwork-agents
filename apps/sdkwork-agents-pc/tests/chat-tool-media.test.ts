import assert from 'node:assert/strict';
import test from 'node:test';

import {
  extractToolMedia,
  toolMediaKind,
  toolProgressKey,
} from '../packages/sdkwork-agents-pc-chat/src/services/toolMedia';

test('toolMediaKind classifies the default MCP media tool set', () => {
  assert.equal(toolMediaKind('mcp__generations__image.create'), 'image');
  assert.equal(toolMediaKind('mcp__generations__image.retrieve'), 'image');
  assert.equal(toolMediaKind('mcp__generations__video.create'), 'video');
  assert.equal(toolMediaKind('mcp__generations__speech.create'), 'audio');
  assert.equal(toolMediaKind('mcp__generations__music.create'), 'audio');
  assert.equal(toolMediaKind('audio.speech.create'), 'audio');
  assert.equal(toolMediaKind('sound-effect.generate'), 'audio');
  // Non-media tools carry no renderable media.
  assert.equal(toolMediaKind('mcp__browser__navigate'), null);
  assert.equal(toolMediaKind('intelligence.model.list'), null);
});

test('extractToolMedia parses the generations payload with resource snapshots', () => {
  const payload = JSON.stringify({
    generation: { id: 'gen-1', modality: 'image', status: 'succeeded' },
    results: [
      {
        id: 'r1',
        resultType: 'image',
        resourceSnapshot: { kind: 'image', source: 'provider_asset', url: 'https://cdn/img-1.png' },
      },
      {
        id: 'r2',
        resultType: 'image',
        resourceSnapshot: { kind: 'image', url: 'https://cdn/img-2.png' },
      },
      { id: 'r3', resourceSnapshot: { kind: 'image', url: '' } },
    ],
    mediaUrls: ['https://cdn/img-1.png', 'https://cdn/img-2.png'],
  });
  const media = extractToolMedia('mcp__generations__image.create', payload);
  assert.deepEqual(
    media.map((item) => item.url),
    ['https://cdn/img-1.png', 'https://cdn/img-2.png'],
  );
  assert.ok(media.every((item) => item.kind === 'image'));
});

test('extractToolMedia parses the media-family items payload', () => {
  const payload = JSON.stringify({
    taskId: 'task-1',
    status: 'succeeded',
    items: [
      { kind: 'audio', source: 'provider_asset', url: 'https://cdn/speech-1.mp3' },
      { kind: 'audio', url: 'https://cdn/speech-2.mp3' },
    ],
  });
  const media = extractToolMedia('audio.speech.create', payload);
  assert.deepEqual(
    media.map((item) => item.url),
    ['https://cdn/speech-1.mp3', 'https://cdn/speech-2.mp3'],
  );
  assert.ok(media.every((item) => item.kind === 'audio'));
});

test('extractToolMedia falls back to mediaUrls with the tool-derived kind', () => {
  const payload = JSON.stringify({ generation: { id: 'gen-2' }, mediaUrls: ['https://cdn/clip.mp4'] });
  const media = extractToolMedia('mcp__generations__video.create', payload);
  assert.deepEqual(media, [{ kind: 'video', url: 'https://cdn/clip.mp4' }]);
});

test('extractToolMedia tolerates raw objects, unparsable JSON, and empty payloads', () => {
  const object = { mediaUrls: ['https://cdn/a.png'] };
  assert.deepEqual(extractToolMedia('mcp__generations__image.create', object), [
    { kind: 'image', url: 'https://cdn/a.png' },
  ]);
  assert.deepEqual(extractToolMedia('mcp__generations__image.create', 'not-json'), []);
  assert.deepEqual(extractToolMedia('mcp__generations__image.create', null), []);
  assert.deepEqual(extractToolMedia('mcp__generations__image.create', undefined), []);
  assert.deepEqual(extractToolMedia('mcp__generations__image.create', '{}'), []);
});

test('extractToolMedia never returns media for non-media tools', () => {
  const payload = JSON.stringify({ mediaUrls: ['https://cdn/a.png'] });
  assert.deepEqual(extractToolMedia('mcp__browser__navigate', payload), []);
});

test('extractToolMedia maps music results to audio rendering', () => {
  const payload = JSON.stringify({
    generation: { id: 'gen-3', modality: 'music' },
    results: [
      { resultType: 'music', resourceSnapshot: { kind: 'music', url: 'https://cdn/track.mp3' } },
    ],
  });
  const media = extractToolMedia('mcp__generations__music.retrieve', payload);
  assert.deepEqual(media, [{ kind: 'audio', url: 'https://cdn/track.mp3' }]);
});

test('toolProgressKey returns null for non-media tools', () => {
  assert.equal(toolProgressKey('mcp__generations__image.create'), 'toolProgress.image');
  assert.equal(toolProgressKey('mcp__generations__video.create'), 'toolProgress.video');
  assert.equal(toolProgressKey('mcp__generations__speech.create'), 'toolProgress.audio');
  assert.equal(toolProgressKey('mcp__generations__music.create'), 'toolProgress.audio');
  assert.equal(toolProgressKey('mcp__browser__navigate'), null);
});
