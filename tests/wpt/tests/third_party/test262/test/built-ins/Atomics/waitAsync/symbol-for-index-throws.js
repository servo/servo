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

  2. Let i be ? ValidateAtomicAccess(typedArray, index).

  ValidateAtomicAccess( typedArray, requestIndex )

  2. Let accessIndex be ? ToIndex(requestIndex).

  ToIndex ( value )

  2. Else,
    a. Let integerIndex be ? ToInteger(value).

  ToInteger(value)

  1. Let number be ? ToNumber(argument).

    Symbol --> Throw a TypeError exception.

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
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(Test262Error, function() {
  Atomics.waitAsync(i32a, poisonedValueOf, poisonedValueOf, poisonedValueOf);
}, '`Atomics.waitAsync(i32a, poisonedValueOf, poisonedValueOf, poisonedValueOf)` throws a Test262Error exception');

assert.throws(Test262Error, function() {
  Atomics.waitAsync(i32a, poisonedToPrimitive, poisonedToPrimitive, poisonedToPrimitive);
}, '`Atomics.waitAsync(i32a, poisonedToPrimitive, poisonedToPrimitive, poisonedToPrimitive)` throws a Test262Error exception');

assert.throws(TypeError, function() {
  Atomics.waitAsync(i32a, Symbol('1'), poisonedValueOf, poisonedValueOf);
}, '`Atomics.waitAsync(i32a, Symbol("1"), poisonedValueOf, poisonedValueOf)` throws a TypeError exception');

assert.throws(TypeError, function() {
  Atomics.waitAsync(i32a, Symbol('2'), poisonedToPrimitive, poisonedToPrimitive);
}, '`Atomics.waitAsync(i32a, Symbol("2"), poisonedToPrimitive, poisonedToPrimitive)` throws a TypeError exception');


