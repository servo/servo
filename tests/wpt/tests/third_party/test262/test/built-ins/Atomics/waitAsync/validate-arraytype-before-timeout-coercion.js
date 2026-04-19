// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  TypedArray type is validated before `timeout` argument is coerced.
info: |
  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  1. Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).

  ValidateSharedIntegerTypedArray ( typedArray [ , waitable ] )

  1. If waitable is not present, set waitable to false.
  2. Perform ? RequireInternalSlot(typedArray, [[TypedArrayName]]).
  3. Let typeName be typedArray.[[TypedArrayName]].
  4. Let type be the Element Type value in Table 61 for typeName.
  5. If waitable is true, then
    a. If typeName is not "Int32Array" or "BigInt64Array", throw a TypeError exception.
  6. Else,
    a. If ! IsUnclampedIntegerElementType(type) is false and ! IsBigIntElementType(type) is false, throw a TypeError exception.
  7. Assert: typedArray has a [[ViewedArrayBuffer]] internal slot.
  8. Let buffer be typedArray.[[ViewedArrayBuffer]].
  9. If IsSharedArrayBuffer(buffer) is false, throw a TypeError exception.
  10. Return buffer.

features: [Atomics.waitAsync, Atomics, TypedArray, SharedArrayBuffer]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const timeout = {
  valueOf() {
    throw new Test262Error();
  }
};

const nonSharedArrayTypes = [
  Int8Array, Uint8Array, Int16Array, Uint16Array, Uint32Array,
  Uint8ClampedArray, Float32Array, Float64Array
];

for (const nonSharedArrayType of nonSharedArrayTypes) {
  const typedArray = new nonSharedArrayType(new SharedArrayBuffer(8));
  assert.throws(TypeError, function() {
    Atomics.waitAsync(typedArray, 0, 0, timeout);
  }, '`Atomics.waitAsync(typedArray, 0, 0, timeout)` throws a TypeError exception');
}
