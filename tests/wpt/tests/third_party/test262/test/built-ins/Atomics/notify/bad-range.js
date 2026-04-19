// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Test range checking of Atomics.notify on arrays that allow atomic operations
info: |
  Atomics.notify( typedArray, index, count )

  1. Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).
  ..

includes: [testAtomics.js]
features: [ArrayBuffer, Atomics, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

testWithAtomicsOutOfBoundsIndices(function(IdxGen) {
  assert.throws(RangeError, function() {
    Atomics.notify(i32a, IdxGen(i32a), 0);
  });
});
