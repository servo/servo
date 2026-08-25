// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Throws a TypeError if value arg is a Symbol
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  5. Otherwise, let v be ? ToInt32(value).

  ToInt32(value)

  1.Let number be ? ToNumber(argument).

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
    throw new Test262Error("passing a poisoned object using @@ToPrimitive");
  }
};

assert.throws(Test262Error, function() {
  Atomics.waitAsync(i32a, 0, poisonedValueOf, poisonedValueOf);
}, '`Atomics.waitAsync(i32a, 0, poisonedValueOf, poisonedValueOf)` throws a Test262Error exception');

assert.throws(Test262Error, function() {
  Atomics.waitAsync(i32a, 0, poisonedToPrimitive, poisonedToPrimitive);
}, '`Atomics.waitAsync(i32a, 0, poisonedToPrimitive, poisonedToPrimitive)` throws a Test262Error exception');

assert.throws(TypeError, function() {
  Atomics.waitAsync(i32a, 0, Symbol("foo"), poisonedValueOf);
}, '`Atomics.waitAsync(i32a, 0, Symbol("foo"), poisonedValueOf)` throws a TypeError exception');

assert.throws(TypeError, function() {
  Atomics.waitAsync(i32a, 0, Symbol("foo"), poisonedToPrimitive);
}, '`Atomics.waitAsync(i32a, 0, Symbol("foo"), poisonedToPrimitive)` throws a TypeError exception');
