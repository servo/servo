// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced caches the length getting the array elements.
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  3. Let len be ? LengthOfArrayLike(O).
  ...
  5. Let k be 0.
  6. Repeat, while k < len,
    a. Let Pk be ! ToString(𝔽(k)).
    b. Let kValue be ? Get(O, Pk).
    ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [0, 1, 2];
Object.defineProperty(arr, "0", {
  get() {
    arr.push(10);
    return 0;
  }
});
Object.defineProperty(arr, "2", {
  get() {
    arr.push(11);
    return 2;
  }
});

assert.compareArray(arr.toSpliced(1, 0, 0.5), [0, 0.5, 1, 2]);
