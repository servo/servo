// META: title=IndexedDB: large nested objects are cloned correctly
// META: timeout=long
// META: script=support-promises.js
// META: script=nested-cloning-common.js
// META: global=window,dedicatedworker,sharedworker,serviceworker
'use strict';

cloningTest('large typed array', [
  { type: 'buffer', size: wrapThreshold, seed: 1 },
])

cloningTestWithKeyGenerator('blob with large typed array', [
  {
    blob: { type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-01',
            seed: 1 },
    buffer: { type: 'buffer', size: wrapThreshold, seed: 2 },
  },
]);

cloningTestWithKeyGenerator('array of blobs and large typed arrays', [
  [
    { type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-01', seed: 1 },
    { type: 'buffer', size: wrapThreshold, seed: 2 },
    { type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-03', seed: 3 },
    { type: 'buffer', size: wrapThreshold, seed: 4 },
    { type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-05', seed: 5 },
  ],
]);

cloningTestWithKeyGenerator('object with blobs and large typed arrays', [
  {
    blob: { type: 'blob', size: wrapThreshold,
            mimeType: 'text/x-blink1', seed: 1 },
    more: [
      { type: 'buffer', size: wrapThreshold, seed: 2 },
      { type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink3', seed: 3 },
      { type: 'buffer', size: wrapThreshold, seed: 4 },
    ],
    blob2: { type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink5',
             seed: 5 },
  },
]);
