// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.indexof
description: search element is compared using strict comparing (===)
info: |
  22.2.3.13 %TypedArray%.prototype.indexOf (searchElement [ , fromIndex ] )

  %TypedArray%.prototype.indexOf is a distinct function that implements the same
  algorithm as Array.prototype.indexOf as defined in 22.1.3.12 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.12 Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  ...
  8. Repeat, while k < len
    ...
    b. If kPresent is true, then
      i. Let elementK be ? Get(O, ! ToString(k)).
      ii. Let same be the result of performing Strict Equality Comparison
      searchElement === elementK.
      iii. If same is true, return k.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 0, 1, undefined, NaN]));
  assert.sameValue(sample.indexOf("42"), -1, "'42'");
  assert.sameValue(sample.indexOf([42]), -1, "[42]");
  assert.sameValue(sample.indexOf(42.0), 0, "42.0");
  assert.sameValue(sample.indexOf(-0), 1, "-0");
  assert.sameValue(sample.indexOf(true), -1, "true");
  assert.sameValue(sample.indexOf(false), -1, "false");
  assert.sameValue(sample.indexOf(NaN), -1, "NaN === NaN is false");
  assert.sameValue(sample.indexOf(null), -1, "null");
  assert.sameValue(sample.indexOf(undefined), -1, "undefined");
  assert.sameValue(sample.indexOf(""), -1, "empty string");
});
