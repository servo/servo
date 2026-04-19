// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.some
description: >
  Does not interact over non-integer properties
info: |
  22.2.3.7 %TypedArray%.prototype.some ( callbackfn [ , thisArg ] )

  ...
  6. Repeat, while k < len
    ...
    c. If kPresent is true, then
      i. Let kValue be ? Get(O, Pk).
      ii. Let testResult be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([7n, 8n]));

  var results = [];

  sample.foo = 42;
  sample[Symbol("1")] = 43;

  sample.some(function() {
    results.push(arguments);
  });

  assert.sameValue(results.length, 2, "results.length");

  assert.sameValue(results[0][1], 0, "results[0][1] - key");
  assert.sameValue(results[1][1], 1, "results[1][1] - key");

  assert.sameValue(results[0][0], 7n, "results[0][0] - value");
  assert.sameValue(results[1][0], 8n, "results[1][0] - value");
});
