// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.includes
description: Returns -1 if buffer is detached after ValidateTypedArray
info: |
  %TypedArray%.prototype.includes ( searchElement [ , fromIndex ] )

  The interpretation and use of the arguments of %TypedArray%.prototype.includes are the same as for Array.prototype.includes as defined in 22.1.3.13.

  When the includes method is called with one or two arguments, the following steps are taken:

  Let O be the this value.
  Perform ? ValidateTypedArray(O).
  Let len be O.[[ArrayLength]].
  If len is 0, return false.
  Let n be ? ToIntegerOrInfinity(fromIndex).
  Assert: If fromIndex is undefined, then n is 0.
  If n is +∞, return false.
  Else if n is -∞, set n to 0.
  If n ≥ 0, then
    Let k be n.
  Else,
    Let k be len + n.
    If k < 0, set k to 0.
  Repeat, while k < len,
    Let elementK be the result of ! Get(O, ! ToString(F(k))).
    If SameValueZero(searchElement, elementK) is true, return true.
    Set k to k + 1.
  Return false.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  const sample = new TA(makeCtorArg(1));
  const fromIndex = {
    valueOf() {
      $DETACHBUFFER(sample.buffer);
      return 0;
    }
  };

  assert.sameValue(sample.includes(0n, fromIndex), false);
}, null, null, ["immutable"]);
