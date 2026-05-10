// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.compareexchange
description: Test Atomics.compareExchange on arrays that allow atomic operations.
includes: [testAtomics.js, testTypedArray.js]
features: [ArrayBuffer, Atomics, BigInt, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/
const sab = new SharedArrayBuffer(1024);
const ab = new ArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 2);

testWithBigIntTypedArrayConstructors(function(TA) {
  const view = new TA(sab, 32, 20);
  const control = new TA(ab, 0, 2);
  view[8] = 0n;

  assert.sameValue(
    Atomics.compareExchange(view, 8, 0n, 10n),
    0n,
    'Atomics.compareExchange(view, 8, 0n, 10n) returns 0n'
  );

  assert.sameValue(view[8], 10n, 'The value of view[8] is 10n');
  view[8] = 0n;

  assert.sameValue(
    Atomics.compareExchange(view, 8, 1n, 10n),
    0n,
    'Atomics.compareExchange(view, 8, 1n, 10n) returns 0n'
  );

  assert.sameValue(view[8], 0n, 'The value of view[8] is 0n');
  view[8] = 0n;

  assert.sameValue(
    Atomics.compareExchange(view, 8, 0n, -5n),
    0n,
    'Atomics.compareExchange(view, 8, 0n, -5n) returns 0n'
  );

  control[0] = -5n;

  assert.sameValue(
    view[8],
    control[0],
    'The value of view[8] equals the value of `control[0]` (-5n)'
  );

  view[3] = -5n;
  control[0] = -5n;

  assert.sameValue(
    Atomics.compareExchange(view, 3, -5n, 0n),
    control[0],
    'Atomics.compareExchange(view, 3, -5n, 0n) returns the value of `control[0]` (-5n)'
  );

  assert.sameValue(view[3], 0n, 'The value of view[3] is 0n');
  control[0] = 12345n;
  view[3] = 12345n;

  assert.sameValue(
    Atomics.compareExchange(view, 3, 12345n, 0n),
    control[0],
    'Atomics.compareExchange(view, 3, 12345n, 0n) returns the value of `control[0]` (12345n)'
  );

  assert.sameValue(view[3], 0n, 'The value of view[3] is 0n');
  control[0] = 123456789n;
  view[3] = 123456789n;

  assert.sameValue(
    Atomics.compareExchange(view, 3, 123456789n, 0n),
    control[0],
    'Atomics.compareExchange(view, 3, 123456789n, 0n) returns the value of `control[0]` (123456789n)'
  );

  assert.sameValue(view[3], 0n, 'The value of view[3] is 0n');

  testWithAtomicsInBoundsIndices(function(IdxGen) {
    let Idx = IdxGen(view);
    view.fill(0n);
    Atomics.store(view, Idx, 37n);

    assert.sameValue(
      Atomics.compareExchange(view, Idx, 37n, 0n),
      37n,
      'Atomics.compareExchange(view, Idx, 37n, 0n) returns 37n'
    );
  });
}, null, ["passthrough"]);
