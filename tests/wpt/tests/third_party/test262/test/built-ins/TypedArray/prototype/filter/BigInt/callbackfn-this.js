// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  callbackfn `this` value
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  5. If thisArg was supplied, let T be thisArg; else let T be undefined.
  ...
  9. Repeat, while k < len
    ...
    c. Let selected be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var expected = (function() { return this; })();
var thisArg = {};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(3));

  var results1 = [];

  sample.filter(function() {
    results1.push(this);
  });

  assert.sameValue(results1.length, 3, "results1");
  assert.sameValue(results1[0], expected, "without thisArg - [0]");
  assert.sameValue(results1[1], expected, "without thisArg - [1]");
  assert.sameValue(results1[2], expected, "without thisArg - [2]");

  var results2 = [];

  sample.filter(function() {
    results2.push(this);
  }, thisArg);

  assert.sameValue(results2.length, 3, "results2");
  assert.sameValue(results2[0], thisArg, "using thisArg - [0]");
  assert.sameValue(results2[1], thisArg, "using thisArg - [1]");
  assert.sameValue(results2[2], thisArg, "using thisArg - [2]");
});
