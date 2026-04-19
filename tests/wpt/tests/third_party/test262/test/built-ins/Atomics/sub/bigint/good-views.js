// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.sub
description: Test Atomics.sub on arrays that allow atomic operations
includes: [testAtomics.js, testTypedArray.js]
features: [ArrayBuffer, Atomics, BigInt, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/
// Make it interesting - use non-zero byteOffsets and non-zero indexes.
// In-bounds boundary cases for indexing
// Atomics.store() computes an index from Idx in the same way as other
// Atomics operations, not quite like view[Idx].
const sab = new SharedArrayBuffer(1024);
const ab = new ArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 2);

testWithBigIntTypedArrayConstructors(function(TA) {
  const view = new TA(sab, 32, 20);
  const control = new TA(ab, 0, 2);
  view[8] = 100n;
  assert.sameValue(Atomics.sub(view, 8, 10n), 100n, 'Atomics.sub(view, 8, 10n) returns 100n');
  assert.sameValue(view[8], 90n, 'The value of view[8] is 90n');
  assert.sameValue(Atomics.sub(view, 8, -5n), 90n, 'Atomics.sub(view, 8, -5n) returns 90n');
  assert.sameValue(view[8], 95n, 'The value of view[8] is 95');
  view[3] = -5n;
  control[0] = -5n;

  assert.sameValue(
    Atomics.sub(view, 3, 0n),
    control[0],
    'Atomics.sub(view, 3, 0n) returns the value of `control[0]` (-5n)'
  );

  control[0] = 12345n;
  view[3] = 12345n;

  assert.sameValue(
    Atomics.sub(view, 3, 0n),
    control[0],
    'Atomics.sub(view, 3, 0n) returns the value of `control[0]` (12345n)'
  );

  control[0] = 123456789n;
  view[3] = 123456789n;

  assert.sameValue(
    Atomics.sub(view, 3, 0n),
    control[0],
    'Atomics.sub(view, 3, 0n) returns the value of `control[0]` (123456789n)'
  );

  testWithAtomicsInBoundsIndices(function(IdxGen) {
    let Idx = IdxGen(view);
    view.fill(0n);
    Atomics.store(view, Idx, 37n);
    assert.sameValue(Atomics.sub(view, Idx, 0n), 37n, 'Atomics.sub(view, Idx, 0n) returns 37n');
  });
}, null, ["passthrough"]);
