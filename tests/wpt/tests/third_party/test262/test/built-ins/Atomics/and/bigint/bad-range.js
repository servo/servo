// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.and
description: >
  Test range checking of Atomics.and on arrays that allow atomic operations
includes: [testAtomics.js, testTypedArray.js]
features: [ArrayBuffer, Atomics, BigInt, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/
const buffer = new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 2);

testWithBigIntTypedArrayConstructors(function(TA) {
  const view = new TA(buffer);

  testWithAtomicsOutOfBoundsIndices(function(IdxGen) {
    assert.throws(RangeError, function() {
      Atomics.and(view, IdxGen(view), 10n);
    });
  });
}, null, ["passthrough"]);
