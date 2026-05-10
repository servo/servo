// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.includes
description: returns false if the element is not found
info: |
  22.2.3.13 %TypedArray%.prototype.includes ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.includes is a distinct function that implements the
  same algorithm as Array.prototype.includes as defined in 22.1.3.11 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

    ...
  5. If n ≥ 0, then
    a. Let k be n.
  6. Else n < 0,
    a. Let k be len + n.
    b. If k < 0, let k be 0.
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
    b. If SameValueZero(searchElement, elementK) is true, return true.
    c. Increase k by 1.
  8. Return false.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;

  sample = new TA(makeCtorArg([42n, 43n, 42n, 41n]));
  assert.sameValue(sample.includes(44n), false, "includes(44)");
  assert.sameValue(sample.includes(43n, 2), false, "includes(43, 2)");
  assert.sameValue(sample.includes(42n, 3), false, "includes(42, 3)");
  assert.sameValue(sample.includes(44n, -4), false, "includes(44, -4)");
  assert.sameValue(sample.includes(44n, -5), false, "includes(44, -5)");
  assert.sameValue(sample.includes(42n, -1), false, "includes(42, -1)");
});
