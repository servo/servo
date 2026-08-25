// Copyright (C) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.sub
description: >
  Atomics.sub will operate on TA when TA.buffer is not a SharedArrayBuffer
includes: [testTypedArray.js]
features: [ArrayBuffer, Atomics, BigInt, TypedArray]
---*/
testWithBigIntTypedArrayConstructors((TA, makeCtorArg) => {
  const buffer = makeCtorArg(4);
  const view = new TA(buffer);
  assert.sameValue(Atomics.store(view, 0, 1n), 1n, 'Atomics.store(view, 0, 1n) returns 1n');
  assert.sameValue(Atomics.sub(view, 0, 1n), 1n, 'Atomics.sub(view, 0, 1n) returns 1n');
  assert.sameValue(Atomics.load(view, 0), 0n, 'Atomics.load(view, 0) returns 0n');
}, null, ["arraybuffer"], ["immutable"]);
