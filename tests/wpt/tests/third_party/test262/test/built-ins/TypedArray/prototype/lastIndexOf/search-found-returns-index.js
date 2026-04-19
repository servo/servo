// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.lastindexof
description: returns index for the first found element
info: |
  22.2.3.17 %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.lastIndexOf is a distinct function that implements the
  same algorithm as Array.prototype.lastIndexOf as defined in 22.1.3.15 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.15 Array.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  ...
  5. If n ≥ 0, then
    a. If n is -0, let k be +0; else let k be min(n, len - 1).
  6. Else n < 0,
    a. Let k be len + n.
  7. Repeat, while k ≥ 0
    a. Let kPresent be ? HasProperty(O, ! ToString(k)).
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
  var sample = new TA(makeCtorArg([42, 43, 42, 41]));
  assert.sameValue(sample.lastIndexOf(42), 2, "lastIndexOf(42)");
  assert.sameValue(sample.lastIndexOf(43), 1, "lastIndexOf(43)");
  assert.sameValue(sample.lastIndexOf(41), 3, "lastIndexOf(41)");
  assert.sameValue(sample.lastIndexOf(41, 3), 3, "lastIndexOf(41, 3)");
  assert.sameValue(sample.lastIndexOf(41, 4), 3, "lastIndexOf(41, 4)");
  assert.sameValue(sample.lastIndexOf(43, 1), 1, "lastIndexOf(43, 1)");
  assert.sameValue(sample.lastIndexOf(43, 2), 1, "lastIndexOf(43, 2)");
  assert.sameValue(sample.lastIndexOf(43, 3), 1, "lastIndexOf(43, 3)");
  assert.sameValue(sample.lastIndexOf(43, 4), 1, "lastIndexOf(43, 4)");
  assert.sameValue(sample.lastIndexOf(42, 0), 0, "lastIndexOf(42, 0)");
  assert.sameValue(sample.lastIndexOf(42, 1), 0, "lastIndexOf(42, 1)");
  assert.sameValue(sample.lastIndexOf(42, 2), 2, "lastIndexOf(42, 2)");
  assert.sameValue(sample.lastIndexOf(42, 3), 2, "lastIndexOf(42, 3)");
  assert.sameValue(sample.lastIndexOf(42, 4), 2, "lastIndexOf(42, 4)");
  assert.sameValue(sample.lastIndexOf(42, -4), 0, "lastIndexOf(42, -4)");
  assert.sameValue(sample.lastIndexOf(42, -3), 0, "lastIndexOf(42, -3)");
  assert.sameValue(sample.lastIndexOf(42, -2), 2, "lastIndexOf(42, -2)");
  assert.sameValue(sample.lastIndexOf(42, -1), 2, "lastIndexOf(42, -1)");
  assert.sameValue(sample.lastIndexOf(43, -3), 1, "lastIndexOf(43, -3)");
  assert.sameValue(sample.lastIndexOf(43, -2), 1, "lastIndexOf(43, -2)");
  assert.sameValue(sample.lastIndexOf(43, -1), 1, "lastIndexOf(43, -1)");
  assert.sameValue(sample.lastIndexOf(41, -1), 3, "lastIndexOf(41, -1)");
});
