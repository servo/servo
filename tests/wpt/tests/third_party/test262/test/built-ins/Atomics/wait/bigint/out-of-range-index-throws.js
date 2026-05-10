// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Throws a RangeError if value of index arg is out of range
info: |
  Atomics.wait( typedArray, index, value, timeout )

  2.Let i be ? ValidateAtomicAccess(typedArray, index).
    ...
    2.Let accessIndex be ? ToIndex(requestIndex).
    ...
    5. If accessIndex ≥ length, throw a RangeError exception.
features: [Atomics, BigInt, SharedArrayBuffer, TypedArray]
---*/

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)
);

const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(RangeError, function() {
  Atomics.wait(i64a, Infinity, poisoned, poisoned);
});
assert.throws(RangeError, function() {
  Atomics.wait(i64a, 8, poisoned, poisoned);
});
assert.throws(RangeError, function() {
  Atomics.wait(i64a, 200, poisoned, poisoned);
});
