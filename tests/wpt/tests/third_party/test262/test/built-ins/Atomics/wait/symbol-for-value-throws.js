// Copyright (C) 2018 Amal Hussein.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Throws a TypeError if value arg is a Symbol
info: |
  Atomics.wait( typedArray, index, value, timeout )

  3. Let v be ? ToInt32(value).

  ToInt32(value)

  1.Let number be ? ToNumber(argument).

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
    throw new Test262Error("passing a poisoned object using @@ToPrimitive");
  }
};

assert.throws(Test262Error, function() {
  Atomics.wait(i32a, 0, poisonedValueOf, poisonedValueOf);
});

assert.throws(Test262Error, function() {
  Atomics.wait(i32a, 0, poisonedToPrimitive, poisonedToPrimitive);
});

assert.throws(TypeError, function() {
  Atomics.wait(i32a, 0, Symbol("foo"), poisonedValueOf);
});

assert.throws(TypeError, function() {
  Atomics.wait(i32a, 0, Symbol("foo"), poisonedToPrimitive);
});
