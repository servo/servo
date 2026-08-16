// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: >
  Treats explicit undefined comparefn the same as implicit undefined comparefn
info: |
  %TypedArray%.prototype.sort ( comparefn )

  1. If comparefn is not undefined and IsCallable(comparefn) is false, throw a TypeError exception.
  ...
includes: [compareArray.js, testTypedArray.js]
features: [TypedArray, BigInt]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  let sample = new TA(makeCtorArg([42n, 44n, 46n, 43n, 45n]));
  let explicit = sample.sort(undefined);
  let implicit = sample.sort();

  assert.compareArray(explicit, [42n, 43n, 44n, 45n, 46n], 'The value of `explicit` is [42n, 43n, 44n, 45n, 46n]');
  assert.compareArray(implicit, [42n, 43n, 44n, 45n, 46n], 'The value of `implicit` is [42n, 43n, 44n, 45n, 46n]');
}, null, null, ["immutable"]);
