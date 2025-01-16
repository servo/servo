// META: title=IndexedDB: small nested objects are cloned correctly
// META: global=window,worker
// META: script=resources/support-promises.js
// META: script=resources/nested-cloning-common.js
// META: timeout=long

// Spec: https://w3c.github.io/IndexedDB/#abort-transaction

'use strict';

cloningTestWithKeyGenerator('blob with small typed array', [
  {
    blob: {
      type: 'blob',
      size: wrapThreshold,
      mimeType: 'text/x-blink-01',
      seed: 1
    },
    buffer: {type: 'buffer', size: 64, seed: 2},
  },
]);

cloningTestWithKeyGenerator('blob array', [
  [
    {type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-1', seed: 1},
    {type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-2', seed: 2},
    {type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-3', seed: 3},
  ],
]);

cloningTestWithKeyGenerator('array of blobs and small typed arrays', [
  [
    {type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-01', seed: 1},
    {type: 'buffer', size: 64, seed: 2},
    {type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-03', seed: 3},
    {type: 'buffer', size: 64, seed: 4},
    {type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-05', seed: 5},
  ],
]);
