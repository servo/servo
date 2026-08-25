// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.notify
description: >
  Returns 0, when TA.buffer is not a SharedArrayBuffer
info: |
  Atomics.notify( typedArray, index, count )

  Let buffer be ? ValidateIntegerTypedArray(typedArray, true).
  ...
  If IsSharedArrayBuffer(buffer) is false, return 0.

features: [ArrayBuffer, Atomics, BigInt, TypedArray]
---*/
const i64a = new BigInt64Array(
  new ArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 8)
);

assert.sameValue(Atomics.notify(i64a, 0, 0), 0, 'Atomics.notify(i64a, 0, 0) returns 0');

