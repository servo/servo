// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.exchange
description: Test Atomics.exchange on arrays that allow atomic operations.
includes: [testAtomics.js, testTypedArray.js]
features: [ArrayBuffer, Atomics, BigInt, DataView, SharedArrayBuffer, Symbol, TypedArray]
---*/
const sab = new SharedArrayBuffer(1024);
const ab = new ArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 2);

testWithBigIntTypedArrayConstructors(function(TA) {
  const view = new TA(sab, 32, 20);
  const control = new TA(ab, 0, 2);
  view[8] = 0n;
  assert.sameValue(Atomics.exchange(view, 8, 10n), 0n, 'Atomics.exchange(view, 8, 10n) returns 0n');
  assert.sameValue(view[8], 10n, 'The value of view[8] is 10n');

  assert.sameValue(
    Atomics.exchange(view, 8, -5n),
    10n,
    'Atomics.exchange(view, 8, -5n) returns 10n'
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
    Atomics.exchange(view, 3, 0n),
    control[0],
    'Atomics.exchange(view, 3, 0n) returns the value of `control[0]` (-5n)'
  );

  control[0] = 12345n;
  view[3] = 12345n;

  assert.sameValue(
    Atomics.exchange(view, 3, 0n),
    control[0],
    'Atomics.exchange(view, 3, 0n) returns the value of `control[0]` (12345n)'
  );

  control[0] = 123456789n;
  view[3] = 123456789n;

  assert.sameValue(
    Atomics.exchange(view, 3, 0n),
    control[0],
    'Atomics.exchange(view, 3, 0n) returns the value of `control[0]` (123456789n)'
  );

  testWithAtomicsInBoundsIndices(function(IdxGen) {
    let Idx = IdxGen(view);
    view.fill(0n);
    Atomics.store(view, Idx, 37n);

    assert.sameValue(
      Atomics.exchange(view, Idx, 0n),
      37n,
      'Atomics.exchange(view, Idx, 0n) returns 37n'
    );
  });
}, null, ["passthrough"]);
