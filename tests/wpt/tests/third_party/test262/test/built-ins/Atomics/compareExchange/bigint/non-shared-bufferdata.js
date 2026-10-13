// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.compareexchange
description: >
  Atomics.compareExchange will operate on TA when TA.buffer is not a SharedArrayBuffer
includes: [testTypedArray.js]
features: [ArrayBuffer, Atomics, BigInt, TypedArray]
---*/
testWithBigIntTypedArrayConstructors((TA, makeCtorArg) => {
  const buffer = makeCtorArg(4);
  const view = new TA(buffer);
  assert.sameValue(Atomics.compareExchange(view, 0, 0n, 1n), 0n, 'Atomics.compareExchange(view, 0, 0n, 1n) returns 0n');
  assert.sameValue(Atomics.load(view, 0), 1n, 'Atomics.load(view, 0) returns 1n');
}, null, ["arraybuffer"], ["immutable"]);
