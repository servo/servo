// META: title=IndexedDB: basic objects are cloned correctly
// META: global=window,worker
// META: script=resources/support-promises.js
// META: script=resources/nested-cloning-common.js
// META: timeout=long

// Spec: https://w3c.github.io/IndexedDB/#abort-transaction

'use strict';

cloningTest('small typed array', [
  {type: 'buffer', size: 64, seed: 1},
]);

cloningTest('blob', [
  {type: 'blob', size: wrapThreshold, mimeType: 'text/x-blink-1', seed: 1},
]);
