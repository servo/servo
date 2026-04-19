// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Array.prototype.with does not [[Get]] the value in the replaced position
info: |
  Array.prototype.with ( )

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

var arr = [0, 1, 2, 3];
Object.defineProperty(arr, "2", {
  get() {
    throw new Test262Error("Should not get '2'");
  }
});

var result = arr.with(2, 6);
assert.compareArray(result, [0, 1, 6, 3]);
