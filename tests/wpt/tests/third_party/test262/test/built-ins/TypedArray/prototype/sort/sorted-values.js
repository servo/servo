// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: Sort values to numeric ascending order
info: |
  22.2.3.26 %TypedArray%.prototype.sort ( comparefn )

  When the TypedArray SortCompare abstract operation is called with two
  arguments x and y, the following steps are taken:

  ...
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;

  sample = new TA(makeCtorArg([4, 3, 2, 1])).sort();
  assert(compareArray(sample, [1, 2, 3, 4]), "descending values");

  sample = new TA(makeCtorArg([3, 4, 1, 2])).sort();
  assert(compareArray(sample, [1, 2, 3, 4]), "mixed numbers");

  sample = new TA(makeCtorArg([3, 4, 3, 1, 0, 1, 2])).sort();
  assert(compareArray(sample, [0, 1, 1, 2, 3, 3, 4]), "repeating numbers");
});

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1, 0, -0, 2])).sort();
  assert(compareArray(sample, [-0, 0, 1, 2]), "0s");
}, floatArrayConstructors);

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1, 0, -0, 2])).sort();
  assert(compareArray(sample, [0, 0, 1, 2]), "0s");
}, intArrayConstructors);

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([-4, 3, 4, -3, 2, -2, 1, 0])).sort();
  assert(compareArray(sample, [-4, -3, -2, 0, 1, 2, 3, 4]), "negative values");
}, floatArrayConstructors.concat([Int8Array, Int16Array, Int32Array]));

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;

  sample = new TA(makeCtorArg([0.5, 0, 1.5, 1])).sort();
  assert(compareArray(sample, [0, 0.5, 1, 1.5]), "non integers");

  sample = new TA(makeCtorArg([0.5, 0, 1.5, -0.5, -1, -1.5, 1])).sort();
  assert(compareArray(sample, [-1.5, -1, -0.5, 0, 0.5, 1, 1.5]), "non integers + negatives");

  sample = new TA(makeCtorArg([3, 4, Infinity, -Infinity, 1, 2])).sort();
  assert(compareArray(sample, [-Infinity, 1, 2, 3, 4, Infinity]), "infinities");

}, floatArrayConstructors);
