// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.notify
description: >
  Evaluates index before returning 0, when TA.buffer is not a SharedArrayBuffer
info: |
  Atomics.notify( typedArray, index, count )

    Let buffer be ? ValidateIntegerTypedArray(typedArray, true).
  ...
  Else,
    Let intCount be ? ToInteger(count).
    Let c be max(intCount, 0).
  ...
  If IsSharedArrayBuffer(buffer) is false, return 0.

features: [ArrayBuffer, Atomics, BigInt, TypedArray]
---*/

const i64a = new BigInt64Array(
  new ArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 8)
);

const poisoned = {
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  Atomics.notify(i64a, poisoned, 0);
});
