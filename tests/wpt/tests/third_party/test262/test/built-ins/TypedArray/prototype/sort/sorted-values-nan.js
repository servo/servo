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

  NOTE: Because NaN always compares greater than any other value, NaN property
  values always sort to the end of the result when comparefn is not provided.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;

  sample = new TA(makeCtorArg([2, NaN, NaN, 0, 1])).sort();
  assert.sameValue(sample[0], 0, "#1 [0]");
  assert.sameValue(sample[1], 1, "#1 [1]");
  assert.sameValue(sample[2], 2, "#1 [2]");
  assert.sameValue(sample[3], NaN, "#1 [3]");
  assert.sameValue(sample[4], NaN, "#1 [4]");

  sample = new TA(makeCtorArg([3, NaN, NaN, Infinity, 0, -Infinity, 2])).sort();
  assert.sameValue(sample[0], -Infinity, "#2 [0]");
  assert.sameValue(sample[1], 0, "#2 [1]");
  assert.sameValue(sample[2], 2, "#2 [2]");
  assert.sameValue(sample[3], 3, "#2 [3]");
  assert.sameValue(sample[4], Infinity, "#2 [4]");
  assert.sameValue(sample[5], NaN, "#2 [5]");
  assert.sameValue(sample[6], NaN, "#2 [6]");
}, floatArrayConstructors);
