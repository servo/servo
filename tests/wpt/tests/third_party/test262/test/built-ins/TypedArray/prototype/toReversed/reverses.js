// Copyright (C) 2025 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.toreversed
description: >
  %TypedArray%.prototype.toReversed outputs a reversed copy
info: |
  %TypedArray%.prototype.toReversed ( )

  ...
  6. Repeat, while k < len,
    a. Let from be ! ToString(𝔽(len - k - 1)).
    b. Let Pk be ! ToString(𝔽(k)).
    c. Let fromValue be ! Get(O, from).
    d. Perform ! Set(A, Pk, fromValue, true).
    e. Set k to k + 1.
  ...
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors(TA => {
    assert.compareArray(new TA([]).toReversed(), []);
    assert.compareArray(new TA([1]).toReversed(), [1]);
    assert.compareArray(new TA([1, 2, 3, 4]).toReversed(), [4, 3, 2, 1]);
});
