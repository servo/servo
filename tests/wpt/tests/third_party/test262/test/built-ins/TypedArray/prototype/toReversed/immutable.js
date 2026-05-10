// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.toreversed
description: >
  %TypedArray%.prototype.toReversed does not mutate its this value
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg([0, 1, 2]));
  ta.toReversed();

  assert.compareArray(ta, [0, 1, 2]);
  assert.notSameValue(ta.toReversed(), ta);
});
