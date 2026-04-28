// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Primitive `array` argument is coerced to an object.
info: |
  %TypedArray%.prototype.set ( typedArray [ , offset ] )

  1. Assert: array is any ECMAScript language value other than an Object
  with a [[TypedArrayName]] internal slot. If it is such an Object,
  the definition in 22.2.3.23.2 applies.
  [...]
  14. Let src be ? ToObject(array).
  15. Let srcLength be ? LengthOfArrayLike(src).
  [...]
  19. Let limit be targetByteIndex + targetElementSize × srcLength.
  20. Repeat, while targetByteIndex < limit,
    a. Let Pk be ! ToString(k).
    b. Let value be ? Get(src, Pk).
    c. If target.[[ContentType]] is BigInt, set value to ? ToBigInt(value).
    [...]
    f. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType, value, true, Unordered).
    [...]
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray, Symbol]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta1 = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  ta1.set("567");
  assert.compareArray(ta1, [5n, 6n, 7n, 4n], "string");

  var ta2 = new TA(makeCtorArg([1n, 2n, 3n]));
  ta2.set(-10, 2);
  assert.compareArray(ta2, [1n, 2n, 3n], "number");

  var ta3 = new TA(makeCtorArg([1n]));
  ta3.set(false);
  assert.compareArray(ta3, [1n], "boolean");

  var ta4 = new TA(makeCtorArg([1n, 2n]));
  ta4.set(Symbol("desc"), 0);
  assert.compareArray(ta4, [1n, 2n], "symbol");

  var ta5 = new TA(makeCtorArg([1n, 2n]));
  ta5.set(4n, 1);
  assert.compareArray(ta5, [1n, 2n], "bigint");
});
