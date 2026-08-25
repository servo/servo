// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: Use internal ArrayLength instead of getting a length property
info: |
  22.2.3.26 %TypedArray%.prototype.sort ( comparefn )

  ...
  3. Let len be the value of obj's [[ArrayLength]] internal slot.
includes: [testTypedArray.js, compareArray.js]
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
  var sample = new TA(makeCtorArg([42n, 42n, 42n]));
  getCalls = 0;

  Object.defineProperty(TA.prototype, "length", desc);
  Object.defineProperty(sample, "length", desc);

  var result = sample.sort();

  assert.sameValue(getCalls, 0, "ignores length properties");
  assert(
    compareArray(result, sample),
    "result is not affected by custom length"
  );
}, null, ["passthrough"]);
