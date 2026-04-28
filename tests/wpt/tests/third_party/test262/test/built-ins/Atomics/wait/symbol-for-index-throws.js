// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Throws a TypeError if index arg can not be converted to an Integer
info: |
  Atomics.wait( typedArray, index, value, timeout )

  2. Let i be ? ValidateAtomicAccess(typedArray, index).

  ValidateAtomicAccess( typedArray, requestIndex )

  2. Let accessIndex be ? ToIndex(requestIndex).

  ToIndex ( value )

  2. Else,
    a. Let integerIndex be ? ToInteger(value).

  ToInteger(value)

  1. Let number be ? ToNumber(argument).

    Symbol --> Throw a TypeError exception.

features: [Atomics, SharedArrayBuffer, Symbol, Symbol.toPrimitive, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

const poisonedValueOf = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

const poisonedToPrimitive = {
  [Symbol.toPrimitive]: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(Test262Error, function() {
  Atomics.wait(i32a, poisonedValueOf, poisonedValueOf, poisonedValueOf);
});

assert.throws(Test262Error, function() {
  Atomics.wait(i32a, poisonedToPrimitive, poisonedToPrimitive, poisonedToPrimitive);
});

assert.throws(TypeError, function() {
  Atomics.wait(i32a, Symbol('foo'), poisonedValueOf, poisonedValueOf);
});

assert.throws(TypeError, function() {
  Atomics.wait(i32a, Symbol('foo'), poisonedToPrimitive, poisonedToPrimitive);
});
