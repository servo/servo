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
    [...]
    d. Otherwise, set value to ? ToNumber(value).
    [...]
    f. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType, value, true, Unordered).
    [...]
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, Symbol]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta1 = new TA(makeCtorArg([1, 2, 3, 4, 5]));
  ta1.set("678", 1);
  assert.compareArray(ta1, [1, 6, 7, 8, 5], "string");

  var ta2 = new TA(makeCtorArg([1, 2, 3]));
  ta2.set(0);
  assert.compareArray(ta2, [1, 2, 3], "number");

  var ta3 = new TA(makeCtorArg([1, 2, 3]));
  ta3.set(true, 2);
  assert.compareArray(ta3, [1, 2, 3], "boolean");

  var ta4 = new TA(makeCtorArg([1]));
  ta4.set(Symbol());
  assert.compareArray(ta4, [1], "symbol");
}, null, null, ["immutable"]);
