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
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;

  sample = new TA(makeCtorArg([4n, 3n, 2n, 1n])).sort();
  assert(compareArray(sample, [1n, 2n, 3n, 4n]), "descending values");

  sample = new TA(makeCtorArg([3n, 4n, 1n, 2n])).sort();
  assert(compareArray(sample, [1n, 2n, 3n, 4n]), "mixed numbers");

  sample = new TA(makeCtorArg([3n, 4n, 3n, 1n, 0n, 1n, 2n])).sort();
  assert(compareArray(sample, [0n, 1n, 1n, 2n, 3n, 3n, 4n]), "repeating numbers");
});

var sample = new BigInt64Array([-4n, 3n, 4n, -3n, 2n, -2n, 1n, 0n]).sort();
assert(compareArray(sample, [-4n, -3n, -2n, 0n, 1n, 2n, 3n, 4n]), "negative values");
