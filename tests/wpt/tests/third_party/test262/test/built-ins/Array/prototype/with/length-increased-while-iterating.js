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

var arr = [0, 1, 2];
Object.defineProperty(arr, "0", {
  get() {
    arr.push(4);
    return 0;
  }
});

assert.compareArray(arr.with(1, 4), [0, 4, 2]);
