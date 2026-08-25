// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Array.prototype.with does not preserve holes in the array
info: |
  Array.prototype.with ( )

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

var arr = [0, /* hole */, 2, /* hole */, 4];
Array.prototype[3] = 3;

var result = arr.with(2, 6);
assert.compareArray(result, [0, undefined, 6, 3, 4]);
assert(result.hasOwnProperty(1));
assert(result.hasOwnProperty(3));
