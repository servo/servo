// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced caches the length getting the array elements.
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...
  11. Let newLen be len + insertCount - actualDeleteCount.
  ...
  13. Let k be 0.
  14. Repeat, while k < actualStart,
    a. Let Pk be ! ToString(𝔽(k)).
    b. Let kValue be ? Get(O, Pk).
    c. Perform ? CreateDataPropertyOrThrow(A, Pk, kValue).
    d. Set k to k + 1.
  ...
  16. Repeat, while k < newLen,
    a. Let Pk be ! ToString(𝔽(k)).
    b. Let from be ! ToString(𝔽(k + actualDeleteCount - insertCount)).
    c. Let fromValue be ? Get(O, from).
    d. Perform ? CreateDataPropertyOrThrow(A, Pk, fromValue).
    e. Set k to k + 1.
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [0, 1, 2, 3, 4, 5];
Array.prototype[3] = 6;

Object.defineProperty(arr, "2", {
  get() {
    arr.length = 1;
    return 2;
  }
});

assert.compareArray(arr.toSpliced(0, 0), [0, 1, 2, 6, undefined, undefined]);
