// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.waitasync
description: >
  Throws a TypeError if typedArray arg is not an Object
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  1. Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).

  ValidateSharedIntegerTypedArray ( typedArray [ , waitable ] )

  2. Perform ? RequireInternalSlot(typedArray, [[TypedArrayName]]).

  RequireInternalSlot ( O, internalSlot )

  1. If Type(O) is not Object, throw a TypeError exception.
  2. If O does not have an internalSlot internal slot, throw a TypeError exception.

features: [Atomics.waitAsync, Symbol, arrow-function, Atomics]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const poisoned = {
  valueOf() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, () => {
  Atomics.waitAsync(null, poisoned, poisoned, poisoned);
}, '`Atomics.waitAsync(null, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Atomics.waitAsync(undefined, poisoned, poisoned, poisoned);
}, '`Atomics.waitAsync(undefined, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Atomics.waitAsync(true, poisoned, poisoned, poisoned);
}, '`Atomics.waitAsync(true, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Atomics.waitAsync(false, poisoned, poisoned, poisoned);
}, '`Atomics.waitAsync(false, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Atomics.waitAsync('***string***', poisoned, poisoned, poisoned);
}, '`Atomics.waitAsync("***string***", poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Atomics.waitAsync(Number.NEGATIVE_INFINITY, poisoned, poisoned, poisoned);
}, '`Atomics.waitAsync(Number.NEGATIVE_INFINITY, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Atomics.waitAsync(Symbol('***symbol***'), poisoned, poisoned, poisoned);
}, '`Atomics.waitAsync(Symbol("***symbol***"), poisoned, poisoned, poisoned)` throws a TypeError exception');

