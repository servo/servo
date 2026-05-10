// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced gets array elements one at a time.
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  16. Repeat, while i < actualStart,
    a. Let Pi be ! ToString(𝔽(i)).
    b. Let iValue be ? Get(O, Pi).
    c. Perform ! CreateDataPropertyOrThrow(A, Pi, iValue).
    d. Set i to i + 1.
  ...
  18. Repeat, while i < newLen,
    a. Let Pi be ! ToString(𝔽(i)).
    b. Let from be ! ToString(𝔽(r)).
    c. Let fromValue be ? Get(O, from).
    d. Perform ! CreateDataPropertyOrThrow(A, Pi, fromValue).
    e. Set i to i + 1.
    f. Set r to r + 1.
  ...

features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [0, 1, 2, 3];
var zerothElementStorage = arr[0];
Object.defineProperty(arr, "0", {
  get() {
    arr[1] = 42;
    return zerothElementStorage;
  },
  set(v) {
    zerothElementStorage = v;
  }
});
Object.defineProperty(arr, "2", {
  get() {
    arr[0] = 17;
    arr[3] = 37;
    return 2;
  }
});

assert.compareArray(arr.toSpliced(1, 0, 0.5), [0, 0.5, 42, 2, 37]);
