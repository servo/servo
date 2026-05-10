// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Test range checking of Atomics.wait on arrays that allow atomic operations
info: |
  Atomics.wait( typedArray, index, value, timeout )

  1. Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).
  ...

includes: [testAtomics.js]
features: [ArrayBuffer, Atomics, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 8)
);

testWithAtomicsOutOfBoundsIndices(function(IdxGen) {
  assert.throws(RangeError, function() {
    Atomics.wait(i32a, IdxGen(i32a), 0, 0);
  });
});
