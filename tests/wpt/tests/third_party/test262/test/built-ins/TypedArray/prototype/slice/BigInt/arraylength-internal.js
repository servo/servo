// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: Use internal ArrayLength instead of getting a length property
info: |
  22.2.3.24 %TypedArray%.prototype.slice ( start, end )

  ...
  3. Let len be the value of O's [[ArrayLength]] internal slot.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var getCalls = 0;
var desc = {
  get: function getLen() {
    getCalls++;
    return 0;
  }
};

Object.defineProperty(TypedArray.prototype, "length", desc);

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n]));

  Object.defineProperty(TA.prototype, "length", desc);
  Object.defineProperty(sample, "length", desc);

  var result = sample.slice();

  assert.sameValue(getCalls, 0, "ignores length properties");
  assert.sameValue(result[0], 42n);
  assert.sameValue(result[1], 43n);
  assert.sameValue(result.hasOwnProperty(2), false);
}, null, ["passthrough"]);
