// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Throws a RangeError is index < 0
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  1. Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).
  2. Let i be ? ValidateAtomicAccess(typedArray, index).

features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

const poisoned = {
  valueOf() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(RangeError, function() {
  Atomics.waitAsync(i32a, -Infinity, poisoned, poisoned);
}, '`Atomics.waitAsync(i32a, -Infinity, poisoned, poisoned)` throws a RangeError exception');
assert.throws(RangeError, function() {
  Atomics.waitAsync(i32a, -7.999, poisoned, poisoned);
}, '`Atomics.waitAsync(i32a, -7.999, poisoned, poisoned)` throws a RangeError exception');
assert.throws(RangeError, function() {
  Atomics.waitAsync(i32a, -1, poisoned, poisoned);
}, '`Atomics.waitAsync(i32a, -1, poisoned, poisoned)` throws a RangeError exception');
assert.throws(RangeError, function() {
  Atomics.waitAsync(i32a, -300, poisoned, poisoned);
}, '`Atomics.waitAsync(i32a, -300, poisoned, poisoned)` throws a RangeError exception');
