// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.waitasync
description: >
  Throws a TypeError if typedArray.buffer is not a SharedArrayBuffer
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  1. Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).

  ValidateSharedIntegerTypedArray ( typedArray [ , waitable ] )

  5. If waitable is true, then
    a. If typeName is not "BigInt64Array" or "BigInt64Array", throw a TypeError exception.

features: [Atomics.waitAsync, ArrayBuffer, Atomics, TypedArray, BigInt, arrow-function]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const i64a = new BigInt64Array(new ArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4));

const poisoned = {
  valueOf() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, () => {
  Atomics.waitAsync(i64a, 0, 0n, 0);
}, '`Atomics.waitAsync(i64a, 0, 0n, 0)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Atomics.waitAsync(i64a, poisoned, poisoned, poisoned);
}, '`Atomics.waitAsync(i64a, poisoned, poisoned, poisoned)` throws a TypeError exception');
