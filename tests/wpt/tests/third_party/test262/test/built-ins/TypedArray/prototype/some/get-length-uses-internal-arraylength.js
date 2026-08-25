// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.some
description: Get "length" uses internal ArrayLength
info: |
  22.2.3.25 %TypedArray%.prototype.some ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.some is a distinct function that implements the same
  algorithm as Array.prototype.some as defined in 22.1.3.24 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.24 Array.prototype.some ( callbackfn [ , thisArg ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? ToLength(? Get(O, "length")).
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
  var calls = 0;

  Object.defineProperty(TA.prototype, "length", desc);
  Object.defineProperty(sample, "length", desc);

  sample.some(function() {
    calls++;
  });

  assert.sameValue(getCalls, 0, "ignores length properties");
  assert.sameValue(calls, 2, "iterations are not affected by custom length");
}, null, ["passthrough"]);
