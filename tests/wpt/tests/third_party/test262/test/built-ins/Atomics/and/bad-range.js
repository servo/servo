// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.and
description: >
  Test range checking of Atomics.and on arrays that allow atomic operations
includes: [testAtomics.js, testTypedArray.js]
features: [ArrayBuffer, Atomics, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/

const buffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2);
const views = nonClampedIntArrayConstructors.slice();

testWithTypedArrayConstructors(function(TA) {
  const view = new TA(buffer);
  testWithAtomicsOutOfBoundsIndices(function(IdxGen) {
    assert.throws(RangeError, function() {
      Atomics.and(view, IdxGen(view), 10);
    });
  });
}, views);
