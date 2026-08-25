// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  Unreachable abrupt from Get(O, "length") as [[ArrayLength]] is returned.
info: |
  22.2.3.5 %TypedArray%.prototype.copyWithin (target, start [ , end ] )

  %TypedArray%.prototype.copyWithin is a distinct function that implements the
  same algorithm as Array.prototype.copyWithin as defined in 22.1.3.3 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length" and the actual copying of values in step 12
  must be performed in a manner that preserves the bit-level encoding of the
  source data.

  ...

  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

Object.defineProperty(TypedArray.prototype, "length", {
  get: function() {
    throw new Test262Error();
  }
});

testWithTypedArrayConstructors(function(TA) {
  Object.defineProperty(TA.prototype, "length", {
    get: function() {
      throw new Test262Error();
    }
  });

  var sample = new TA();
  Object.defineProperty(sample, "length", {
    get: function() {
      throw new Test262Error();
    }
  });

  assert.sameValue(sample.copyWithin(0, 0), sample);
}, null, ["passthrough"]);
