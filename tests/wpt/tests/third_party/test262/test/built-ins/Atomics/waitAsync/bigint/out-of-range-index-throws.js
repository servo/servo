// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Throws a RangeError if value of index arg is out of range
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  2. Let i be ? ValidateAtomicAccess(typedArray, index).

    ...
    2.Let accessIndex be ? ToIndex(requestIndex).
    ...
    5. If accessIndex ≥ length, throw a RangeError exception.
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, BigInt]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)
);

const poisoned = {
  valueOf() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(RangeError, function() {
  Atomics.waitAsync(i64a, Infinity, poisoned, poisoned);
}, '`Atomics.waitAsync(i64a, Infinity, poisoned, poisoned)` throws a RangeError exception');
assert.throws(RangeError, function() {
  Atomics.waitAsync(i64a, -1, poisoned, poisoned);
}, '`Atomics.waitAsync(i64a, -1, poisoned, poisoned)` throws a RangeError exception');
assert.throws(RangeError, function() {
  Atomics.waitAsync(i64a, 4, poisoned, poisoned);
}, '`Atomics.waitAsync(i64a, 4, poisoned, poisoned)` throws a RangeError exception');
assert.throws(RangeError, function() {
  Atomics.waitAsync(i64a, 200, poisoned, poisoned);
}, '`Atomics.waitAsync(i64a, 200, poisoned, poisoned)` throws a RangeError exception');

