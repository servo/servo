// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Throws a RangeError is index < 0
info: |
  Atomics.wait( typedArray, index, value, timeout )

  2.Let i be ? ValidateAtomicAccess(typedArray, index).
    ...
      2.Let accessIndex be ? ToIndex(requestIndex).
        ...
        2.b If integerIndex < 0, throw a RangeError exception
features: [Atomics, BigInt, SharedArrayBuffer, TypedArray]
---*/

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 8)
);
const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(RangeError, function() {
  Atomics.wait(i64a, -Infinity, poisoned, poisoned);
});
assert.throws(RangeError, function() {
  Atomics.wait(i64a, -7.999, poisoned, poisoned);
});
assert.throws(RangeError, function() {
  Atomics.wait(i64a, -1, poisoned, poisoned);
});
assert.throws(RangeError, function() {
  Atomics.wait(i64a, -300, poisoned, poisoned);
});
