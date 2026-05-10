// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.includes
description: search element is compared using SameValueZero
info: |
  22.2.3.13 %TypedArray%.prototype.includes ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.includes is a distinct function that implements the
  same algorithm as Array.prototype.includes as defined in 22.1.3.11 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
    b. If SameValueZero(searchElement, elementK) is true, return true.
    c. Increase k by 1.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 0, 1, undefined]));
  assert.sameValue(sample.includes(), false, "no arg");
  assert.sameValue(sample.includes(undefined), false, "undefined");
  assert.sameValue(sample.includes("42"), false, "'42'");
  assert.sameValue(sample.includes([42]), false, "[42]");
  assert.sameValue(sample.includes(42.0), true, "42.0");
  assert.sameValue(sample.includes(-0), true, "-0");
  assert.sameValue(sample.includes(true), false, "true");
  assert.sameValue(sample.includes(false), false, "false");
  assert.sameValue(sample.includes(null), false, "null");
  assert.sameValue(sample.includes(""), false, "empty string");
});

testWithTypedArrayConstructors(function(FloatArray, makeCtorArg) {
  var sample = new FloatArray(makeCtorArg([42, 0, 1, undefined, NaN]));
  assert.sameValue(sample.includes(NaN), true, "NaN");
}, floatArrayConstructors);
