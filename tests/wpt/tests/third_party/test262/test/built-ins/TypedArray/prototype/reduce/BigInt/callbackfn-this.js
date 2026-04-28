// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  callbackfn `this` value
info: |
  22.2.3.20 %TypedArray%.prototype.reduce ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduce is a distinct function that implements the same
  algorithm as Array.prototype.reduce as defined in 22.1.3.19 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.19 Array.prototype.reduce ( callbackfn [ , initialValue ] )

  ...
  8. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      i. Let accumulator be ? Call(callbackfn, undefined, « accumulator, kValue,
      k, O »).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var expected = (function() { return this; })();

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(3));

  var results = [];

  sample.reduce(function() {
    results.push(this);
  }, 0);

  assert.sameValue(results.length, 3, "results.length");
  assert.sameValue(results[0], expected, "[0]");
  assert.sameValue(results[1], expected, "[1]");
  assert.sameValue(results[2], expected, "[2]");
});
