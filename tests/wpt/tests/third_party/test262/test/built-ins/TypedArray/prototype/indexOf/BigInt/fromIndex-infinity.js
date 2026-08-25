// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.indexof
description: handle Infinity values for fromIndex
info: |
  22.2.3.13 %TypedArray%.prototype.indexOf (searchElement [ , fromIndex ] )

  %TypedArray%.prototype.indexOf is a distinct function that implements the same
  algorithm as Array.prototype.indexOf as defined in 22.1.3.12 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.12 Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  ...
  6. If n ≥ 0, then
    a. If n is -0, let k be +0; else let k be n.
  7. Else n < 0,
    a. Let k be len + n.
    b. If k < 0, let k be 0.
  8. Repeat, while k < len
    a. Let kPresent be ? HasProperty(O, ! ToString(k)).
    b. If kPresent is true, then
      i. Let elementK be ? Get(O, ! ToString(k)).
      ii. Let same be the result of performing Strict Equality Comparison
      searchElement === elementK.
      iii. If same is true, return k.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n, 43n, 41n]));

  assert.sameValue(sample.indexOf(43n, Infinity), -1, "indexOf(43, Infinity)");
  assert.sameValue(sample.indexOf(43n, -Infinity), 1, "indexOf(43, -Infinity)");
});
