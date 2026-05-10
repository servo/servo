// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.every
description: >
  callbackfn `this` value
info: |
  22.2.3.7 %TypedArray%.prototype.every ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.every is a distinct function that implements the same
  algorithm as Array.prototype.every as defined in 22.1.3.5 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.5 Array.prototype.every ( callbackfn [ , thisArg ] )

  ...
  4. If thisArg was supplied, let T be thisArg; else let T be undefined.
  ...
  6. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      ii. Let testResult be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var expected = (function() { return this; })();
var thisArg = {};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(3));

  var results1 = [];

  sample.every(function() {
    results1.push(this);
    return true;
  });

  assert.sameValue(results1.length, 3, "results1");
  assert.sameValue(results1[0], expected, "without thisArg - [0]");
  assert.sameValue(results1[1], expected, "without thisArg - [1]");
  assert.sameValue(results1[2], expected, "without thisArg - [2]");

  var results2 = [];

  sample.every(function() {
    results2.push(this);
    return true;
  }, thisArg);

  assert.sameValue(results2.length, 3, "results2");
  assert.sameValue(results2[0], thisArg, "using thisArg - [0]");
  assert.sameValue(results2[1], thisArg, "using thisArg - [1]");
  assert.sameValue(results2[2], thisArg, "using thisArg - [2]");
});
