// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.lastindexof
description: search element is compared using strict comparing (===)
info: |
  22.2.3.17 %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.lastIndexOf is a distinct function that implements the
  same algorithm as Array.prototype.lastIndexOf as defined in 22.1.3.15 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.15 Array.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  ...
  7. Repeat, while k ≥ 0
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
  var sample = new TA(makeCtorArg([42, undefined, NaN, 0, 1]));
  assert.sameValue(sample.lastIndexOf("42"), -1, "'42'");
  assert.sameValue(sample.lastIndexOf([42]), -1, "[42]");
  assert.sameValue(sample.lastIndexOf(42.0), 0, "42.0");
  assert.sameValue(sample.lastIndexOf(-0), 3, "-0");
  assert.sameValue(sample.lastIndexOf(true), -1, "true");
  assert.sameValue(sample.lastIndexOf(false), -1, "false");
  assert.sameValue(sample.lastIndexOf(NaN), -1, "NaN === NaN is false");
  assert.sameValue(sample.lastIndexOf(null), -1, "null");
  assert.sameValue(sample.lastIndexOf(undefined), -1, "undefined");
  assert.sameValue(sample.lastIndexOf(""), -1, "empty string");
});
