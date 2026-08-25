// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Test range checking of Atomics.waitAsync on arrays that allow atomic operations
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  ...
  2. Let i be ? ValidateAtomicAccess(typedArray, index).
  ...

includes: [testAtomics.js]
features: [Atomics.waitAsync, Atomics, SharedArrayBuffer, ArrayBuffer, DataView, Symbol, TypedArray]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 8)
);

testWithAtomicsOutOfBoundsIndices(function(IdxGen) {
  assert.throws(RangeError, function() {
    Atomics.waitAsync(i32a, IdxGen(i32a), 0, 0);
  }, '`Atomics.waitAsync(i32a, IdxGen(i32a), 0, 0)` throws a RangeError exception');
});


