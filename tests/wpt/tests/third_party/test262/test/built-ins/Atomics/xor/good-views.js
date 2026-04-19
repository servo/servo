// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.xor
description: Test Atomics.xor on arrays that allow atomic operations
includes: [testAtomics.js, testTypedArray.js]
features: [ArrayBuffer, Atomics, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/

var sab = new SharedArrayBuffer(1024);
var ab = new ArrayBuffer(16);
var views = nonClampedIntArrayConstructors.slice();

testWithTypedArrayConstructors(function(TA) {
  // Make it interesting - use non-zero byteOffsets and non-zero indexes.

  var view = new TA(sab, 32, 20);
  var control = new TA(ab, 0, 2);

  view[8] = 0x33333333;
  control[0] = 0x33333333;
  assert.sameValue(Atomics.xor(view, 8, 0x55555555), control[0],
    'Atomics.xor(view, 8, 0x55555555) returns the value of `control[0]` (0x33333333)');

  control[0] = 0x66666666;
  assert.sameValue(
    view[8],
    control[0],
    'The value of view[8] equals the value of `control[0]` (0x66666666)'
  );
  assert.sameValue(Atomics.xor(view, 8, 0xF0F0F0F0), control[0],
    'Atomics.xor(view, 8, 0xF0F0F0F0) returns the value of `control[0]` (0x66666666)');

  control[0] = 0x96969696;
  assert.sameValue(
    view[8],
    control[0],
    'The value of view[8] equals the value of `control[0]` (0x96969696)'
  );

  view[3] = -5;
  control[0] = -5;
  assert.sameValue(Atomics.xor(view, 3, 0), control[0],
    'Atomics.xor(view, 3, 0) returns the value of `control[0]` (-5)');
  assert.sameValue(
    view[3],
    control[0],
    'The value of view[3] equals the value of `control[0]` (-5)'
  );

  control[0] = 12345;
  view[3] = 12345;
  assert.sameValue(Atomics.xor(view, 3, 0), control[0],
    'Atomics.xor(view, 3, 0) returns the value of `control[0]` (12345)');
  assert.sameValue(
    view[3],
    control[0],
    'The value of view[3] equals the value of `control[0]` (12345)'
  );

  // And again
  control[0] = 123456789;
  view[3] = 123456789;
  assert.sameValue(Atomics.xor(view, 3, 0), control[0],
    'Atomics.xor(view, 3, 0) returns the value of `control[0]` (123456789)');
  assert.sameValue(
    view[3],
    control[0],
    'The value of view[3] equals the value of `control[0]` (123456789)'
  );

  // In-bounds boundary cases for indexing
  testWithAtomicsInBoundsIndices(function(IdxGen) {
    let Idx = IdxGen(view);
    view.fill(0);
    // Atomics.store() computes an index from Idx in the same way as other
    // Atomics operations, not quite like view[Idx].
    Atomics.store(view, Idx, 37);
    assert.sameValue(Atomics.xor(view, Idx, 0), 37, 'Atomics.xor(view, Idx, 0) returns 37');
  });
}, views);
