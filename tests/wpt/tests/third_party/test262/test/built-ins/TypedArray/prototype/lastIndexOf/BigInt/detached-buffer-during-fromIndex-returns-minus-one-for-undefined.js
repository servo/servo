// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.lastindexof
description: Returns -1 if buffer is detached after ValidateTypedArray
info: |
  %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  The interpretation and use of the arguments of %TypedArray%.prototype.lastIndexOf are the same as for Array.prototype.lastIndexOf as defined in 22.1.3.17.

  When the lastIndexOf method is called with one or two arguments, the following steps are taken:

  Let O be the this value.
  Perform ? ValidateTypedArray(O).
  Let len be O.[[ArrayLength]].
  If len is 0, return -1F.
  If fromIndex is present, let n be ? ToIntegerOrInfinity(fromIndex); else let n be len - 1.
  If n is -∞, return -1F.
  If n ≥ 0, then
    Let k be min(n, len - 1).
  Else,
    Let k be len + n.
  Repeat, while k ≥ 0,
    Let kPresent be ! HasProperty(O, ! ToString(F(k))).
    If kPresent is true, then
      Let elementK be ! Get(O, ! ToString(F(k))).
      Let same be the result of performing Strict Equality Comparison searchElement === elementK.
      If same is true, return F(k).
    Set k to k - 1.
  Return -1F.

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

  assert.sameValue(sample.lastIndexOf(undefined, fromIndex), -1);
}, null, null, ["immutable"]);
