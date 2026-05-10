// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: Uses internal ArrayLength instead of length property
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  3. Let len be the value of O's [[ArrayLength]] internal slot.
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
  var sample = new TA(makeCtorArg(4));
  var calls = 0;

  Object.defineProperty(TA.prototype, "length", desc);
  Object.defineProperty(sample, "length", desc);

  sample.filter(function() {
    calls++;
  });

  assert.sameValue(getCalls, 0, "ignores length properties");
  assert.sameValue(calls, 4, "interactions are not affected by custom length");
}, null, ["passthrough"]);
