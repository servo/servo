// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Array.prototype.with caches the length getting the array elements.
info: |
  Array.prototype.with ( index, value )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...
  5. Repeat, while k < len
    a. Let Pk be ! ToString(𝔽(k)).
    b. If k is actualIndex, let fromValue be value.
    c. Else, let fromValue be ? Get(O, Pk).
    d. Perform ? CreateDataPropertyOrThrow(A, Pk, fromValue).
    e. Set k to k + 1.
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

Array.prototype[4] = 5;

var arr = Object.defineProperty([0, 1, 2, 3, 4], "1", {
  get() {
    arr.length = 1;
    return 1;
  }
});
assert.compareArray(arr.with(2, 7), [0, 1, 7, undefined, 5]);

arr = Object.defineProperty([0, 1, 2, 3, 4], "1", {
  get() {
    arr.length = 1;
    return 1;
  }
});
assert.compareArray(arr.with(0, 7), [7, 1, undefined, undefined, 5]);
