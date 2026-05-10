// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.compareexchange
description: >
  Test range checking of Atomics.compareExchange on arrays that allow atomic operations
includes: [testAtomics.js, testTypedArray.js]
features: [ArrayBuffer, Atomics, BigInt, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/
const buffer = new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 2);

testWithBigIntTypedArrayConstructors(function(TA) {
  const view = new TA(buffer);

  testWithAtomicsOutOfBoundsIndices(function(IdxGen) {
    assert.throws(RangeError, function() {
      Atomics.compareExchange(view, IdxGen(view), 10, 0n);
    });
  });
}, null, ["passthrough"]);
