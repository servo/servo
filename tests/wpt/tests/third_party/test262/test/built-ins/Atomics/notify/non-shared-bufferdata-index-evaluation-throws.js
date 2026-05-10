// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.notify
description: >
  Evaluates index before returning 0, when TA.buffer is not a SharedArrayBuffer
info: |
  Atomics.notify( typedArray, index, count )

  Let buffer be ? ValidateIntegerTypedArray(typedArray, true).
  Let i be ? ValidateAtomicAccess(typedArray, index).
  ...
  If IsSharedArrayBuffer(buffer) is false, return 0.

features: [ArrayBuffer, Atomics, TypedArray]
---*/

const i32a = new Int32Array(
  new ArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

const poisoned = {
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  Atomics.notify(i32a, poisoned, 0);
});


