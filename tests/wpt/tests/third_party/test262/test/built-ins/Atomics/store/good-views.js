// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.store
description: Test Atomics.store on arrays that allow atomic operations.
includes: [testAtomics.js, testTypedArray.js]
features: [ArrayBuffer, Atomics, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/

const sab = new SharedArrayBuffer(1024);
const ab = new ArrayBuffer(16);
const views = nonClampedIntArrayConstructors.slice();

testWithTypedArrayConstructors(function(TA) {
  // Make it interesting - use non-zero byteOffsets and non-zero indexes.

  const view = new TA(sab, 32, 20);
  const control = new TA(ab, 0, 2);

  const values = [
    10,
    -5,
    12345,
    123456789,
    Math.PI,
    "33",
    {
      valueOf: function() { return 33; }
    },
    undefined
  ];

  for (let i = 0; i < values.length; i++) {
    let val = values[i];
    assert.sameValue(Atomics.store(view, 3, val), ToInteger(val),
      'Atomics.store(view, 3, val) returns ToInteger(val)');

    control[0] = val;
    assert.sameValue(
      view[3],
      control[0],
      'The value of view[3] equals the value of `control[0]` (val)'
    );
  }

  // In-bounds boundary cases for indexing
  testWithAtomicsInBoundsIndices(function(IdxGen) {
    let Idx = IdxGen(view);
    view.fill(0);
    Atomics.store(view, Idx, 37);
    assert.sameValue(Atomics.load(view, Idx), 37, 'Atomics.load(view, Idx) returns 37');
  });
}, views);

function ToInteger(v) {
  v = +v;
  if (isNaN(v)) {
    return 0;
  }
  if (v == 0 || !isFinite(v)) {
    return v;
  }
  if (v < 0) {
    return -Math.floor(Math.abs(v));
  }
  return Math.floor(v);
}
