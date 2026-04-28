// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  TypedArray type is validated before `value` argument is coerced.
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

includes: [testTypedArray.js]
features: [Atomics.waitAsync, Atomics, TypedArray, SharedArrayBuffer]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const value = {
  valueOf() {
    throw new Test262Error();
  }
};

var nonSharedArrayTypes = typedArrayConstructors.filter(function(TA) { return TA !== Int32Array; });

for (const nonSharedArrayType of nonSharedArrayTypes) {
  const typedArray = new nonSharedArrayType(new SharedArrayBuffer(8));
  assert.throws(TypeError, function() {
    Atomics.waitAsync(typedArray, 0, value, 0);
  }, '`Atomics.waitAsync(typedArray, 0, value, 0)` throws a TypeError exception');
}
