// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.join
description: Returns single separator if buffer is detached after ValidateTypedArray
info: |
  %TypedArray%.prototype.join ( separator )

  The interpretation and use of the arguments of %TypedArray%.prototype.join are the same as for Array.prototype.join as defined in 22.1.3.15.

  When the join method is called with one argument separator, the following steps are taken:

  Let O be the this value.
  Perform ? ValidateTypedArray(O).
  Let len be O.[[ArrayLength]].
  If separator is undefined, let sep be the single-element String ",".
  Else, let sep be ? ToString(separator).
  Let R be the empty String.
  Let k be 0.
  Repeat, while k < len,
    If k > 0, set R to the string-concatenation of R and sep.
    Let element be ! Get(O, ! ToString(𝔽(k))).
    If element is undefined or null, let next be the empty String; otherwise, let next be ! ToString(element).
    Set R to the string-concatenation of R and next.
    Set k to k + 1.
  Return R.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  const sample = new TA([1n,2n,3n]);
  const separator = {
    toString() {
      $DETACHBUFFER(sample.buffer);
      return ',';
    }
  };

  assert.sameValue(sample.join(separator), ',,');
}, null, ["passthrough"]);
