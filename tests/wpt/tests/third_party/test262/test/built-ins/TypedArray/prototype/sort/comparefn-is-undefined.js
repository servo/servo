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
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  let sample = new TA(makeCtorArg([42, 44, 46, 43, 45]));
  let explicit = sample.sort(undefined);
  let implicit = sample.sort();

  assert.compareArray(explicit, [42, 43, 44, 45, 46], 'The value of `explicit` is [42, 43, 44, 45, 46]');
  assert.compareArray(implicit, [42, 43, 44, 45, 46], 'The value of `implicit` is [42, 43, 44, 45, 46]');
}, null, null, ["immutable"]);
