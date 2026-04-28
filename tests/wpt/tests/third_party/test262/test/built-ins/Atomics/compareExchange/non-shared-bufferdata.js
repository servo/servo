// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.compareexchange
description: >
  Atomics.compareExchange will operate on TA when TA.buffer is not a SharedArrayBuffer
includes: [testTypedArray.js]
features: [ArrayBuffer, Atomics, TypedArray]
---*/
testWithAtomicsFriendlyTypedArrayConstructors(TA => {
  const view = new TA(
    new ArrayBuffer(TA.BYTES_PER_ELEMENT * 4)
  );

  assert.sameValue(Atomics.compareExchange(view, 0, 0, 1), 0, 'Atomics.compareExchange(view, 0, 0, 1) returns 0');
  assert.sameValue(Atomics.load(view, 0), 1, 'Atomics.load(view, 0) returns 1');
}, null, ["passthrough"]);
