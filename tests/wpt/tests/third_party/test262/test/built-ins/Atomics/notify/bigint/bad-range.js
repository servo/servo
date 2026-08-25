// Copyright (C) 2018 Rick Waldron. All rights reserved.
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
features: [ArrayBuffer, Atomics, BigInt, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 8)
);

testWithAtomicsOutOfBoundsIndices(function(IdxGen) {
  assert.throws(RangeError, function() {
    Atomics.notify(i64a, IdxGen(i64a), 0);
  });
});
