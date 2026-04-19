// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.toreversed
description: >
  Array.prototype.toReversed does not preserve holes in the array
info: |
  Array.prototype.toReversed ( )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...
  5. Repeat, while k < len
    a. Let from be ! ToString(𝔽(len - k - 1)).
    ...
    c. Let fromValue be ? Get(O, from).
    d. Perform ? CreateDataPropertyOrThrow(A, Pk, fromValue).
    ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [0, /* hole */, 2, /* hole */, 4];
Array.prototype[3] = 3;

var reversed = arr.toReversed();
assert.compareArray(reversed, [4, 3, 2, undefined, 0]);
assert(reversed.hasOwnProperty(3));
