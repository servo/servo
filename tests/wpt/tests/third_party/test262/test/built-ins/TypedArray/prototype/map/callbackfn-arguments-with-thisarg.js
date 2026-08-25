// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: >
  thisArg does not affect callbackfn arguments
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )

  ...
  8. Repeat, while k < len
    a. Let Pk be ! ToString(k).
    b. Let kValue be ? Get(O, Pk).
    c. Let mappedValue be ? Call(callbackfn, T, « kValue, k, O »).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43, 44]));

  var results = [];
  var thisArg = ["test262", 0, "ecma262", 0];

  sample.map(function() {
    results.push(arguments);
    return 0;
  }, thisArg);

  assert.sameValue(results.length, 3, "results.length");
  assert.sameValue(thisArg.length, 4, "thisArg.length");

  assert.sameValue(results[0].length, 3, "results[0].length");
  assert.sameValue(results[0][0], 42, "results[0][0] - kValue");
  assert.sameValue(results[0][1], 0, "results[0][1] - k");
  assert.sameValue(results[0][2], sample, "results[0][2] - this");

  assert.sameValue(results[1].length, 3, "results[1].length");
  assert.sameValue(results[1][0], 43, "results[1][0] - kValue");
  assert.sameValue(results[1][1], 1, "results[1][1] - k");
  assert.sameValue(results[1][2], sample, "results[1][2] - this");

  assert.sameValue(results[2].length, 3, "results[2].length");
  assert.sameValue(results[2][0], 44, "results[2][0] - kValue");
  assert.sameValue(results[2][1], 2, "results[2][1] - k");
  assert.sameValue(results[2][2], sample, "results[2][2] - this");
});
