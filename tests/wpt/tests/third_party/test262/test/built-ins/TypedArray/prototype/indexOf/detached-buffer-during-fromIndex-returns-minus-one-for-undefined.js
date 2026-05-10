// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.indexof
description: Returns -1 if buffer is detached after ValidateTypedArray
info: |
  %TypedArray%.prototype.indexOf ( searchElement [ , fromIndex ] )

  The interpretation and use of the arguments of %TypedArray%.prototype.indexOf are the same as for Array.prototype.indexOf as defined in 22.1.3.14.

  When the indexOf method is called with one or two arguments, the following steps are taken:

  Let O be the this value.
  Perform ? ValidateTypedArray(O).
  Let len be O.[[ArrayLength]].
  If len is 0, return -1F.
  Let n be ? ToIntegerOrInfinity(fromIndex).
  Assert: If fromIndex is undefined, then n is 0.
  If n is +∞, return -1F.
  Else if n is -∞, set n to 0.
  If n ≥ 0, then
    Let k be n.
  Else,
    Let k be len + n.
  If k < 0, set k to 0.
  Repeat, while k < len,
    Let kPresent be ! HasProperty(O, ! ToString(F(k))).
    If kPresent is true, then
      Let elementK be ! Get(O, ! ToString(F(k))).
      Let same be the result of performing Strict Equality Comparison searchElement === elementK.
      If same is true, return F(k).
    Set k to k + 1.
  Return -1F.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  const sample = new TA(1);
  const fromIndex = {
    valueOf() {
      $DETACHBUFFER(sample.buffer);
      return 0;
    }
  };

  assert.sameValue(sample.indexOf(undefined, fromIndex), -1);
}, null, ["passthrough"]);
