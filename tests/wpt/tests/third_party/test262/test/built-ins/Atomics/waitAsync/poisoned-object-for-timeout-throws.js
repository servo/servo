// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Throws a TypeError if index arg can not be converted to an Integer
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  6. Let q be ? ToNumber(timeout).

  Let primValue be ? ToPrimitive(argument, hint Number).
  Return ? ToNumber(primValue).

features: [Atomics.waitAsync, SharedArrayBuffer, Symbol, Symbol.toPrimitive, TypedArray, computed-property-names, Atomics]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

const poisonedValueOf = {
  valueOf() {
    throw new Test262Error('should not evaluate this code');
  }
};

const poisonedToPrimitive = {
  [Symbol.toPrimitive]() {
    throw new Test262Error('passing a poisoned object using @@ToPrimitive');
  }
};

assert.throws(Test262Error, function() {
  Atomics.waitAsync(i32a, 0, 0, poisonedValueOf);
}, '`Atomics.waitAsync(i32a, 0, 0, poisonedValueOf)` throws a Test262Error exception');

assert.throws(Test262Error, function() {
  Atomics.waitAsync(i32a, 0, 0, poisonedToPrimitive);
}, '`Atomics.waitAsync(i32a, 0, 0, poisonedToPrimitive)` throws a Test262Error exception');
