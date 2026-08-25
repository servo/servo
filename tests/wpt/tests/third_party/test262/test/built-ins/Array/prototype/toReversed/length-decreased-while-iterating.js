// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.toreversed
description: >
  Array.prototype.toReversed caches the length getting the array elements.
info: |
  Array.prototype.toReversed ( )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...
  5. Repeat, while k < len
    ...
    c. Let fromValue be ? Get(O, from).
    ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [0, 1, 2, 3, 4];
Array.prototype[1] = 5;

Object.defineProperty(arr, "3", {
  get() {
    arr.length = 1;
    return 3;
  }
});

assert.compareArray(arr.toReversed(), [4, 3, undefined, 5, 0]);
