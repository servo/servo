// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Does not iterate over non-integer properties
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  9. Repeat, while k < len
    ...
    c. Let selected be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([7, 8]));

  var results = [];

  sample.foo = 42;
  sample[Symbol("1")] = 43;

  sample.filter(function() {
    results.push(arguments);
  });

  assert.sameValue(results.length, 2, "results.length");

  assert.sameValue(results[0][1], 0, "results[0][1] - k");
  assert.sameValue(results[1][1], 1, "results[1][1] - k");

  assert.sameValue(results[0][0], 7, "results[0][0] - kValue");
  assert.sameValue(results[1][0], 8, "results[1][0] - kValue");
});
