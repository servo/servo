// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  %TypedArray%.prototype.with does not mutate its this value
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg([3, 1, 2]));
  ta.with(0, 2);

  assert.compareArray(ta, [3, 1, 2]);
  assert.notSameValue(ta.with(0, 2), ta);
  assert.notSameValue(ta.with(0, 3), ta);
});
