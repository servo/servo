// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.join
description: Get "length" uses internal ArrayLength
info: |
  22.2.3.15 %TypedArray%.prototype.join ( separator )

  %TypedArray%.prototype.join is a distinct function that implements the same
  algorithm as Array.prototype.join as defined in 22.1.3.13 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.13 Array.prototype.join (separator)

  1. Let O be ? ToObject(this value).
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
  5. If len is zero, return the empty String.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var getCalls = 0;
var desc = {
  get: function getLen() {
    getCalls++;
    return 0;
  }
};

Object.defineProperty(TypedArray.prototype, "length", desc);

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43]));

  Object.defineProperty(TA.prototype, "length", desc);
  Object.defineProperty(sample, "length", desc);

  var result = sample.join();

  assert.sameValue(getCalls, 0, "ignores length properties");
  assert.notSameValue(result, "", "result is not affected but custom length 0");
}, null, ["passthrough"]);
