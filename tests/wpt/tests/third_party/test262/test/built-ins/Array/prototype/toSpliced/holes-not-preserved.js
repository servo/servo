// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced does not preserve holes in the array
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

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
includes: [compareArray.js]
features: [change-array-by-copy]
---*/

var arr = [0, /* hole */, 2, /* hole */, 4];
Array.prototype[3] = 3;

var spliced = arr.toSpliced(0, 0);
assert.compareArray(spliced, [0, undefined, 2, 3, 4]);
assert(spliced.hasOwnProperty(1));
assert(spliced.hasOwnProperty(3));

spliced = arr.toSpliced(0, 0, -1);
assert.compareArray(spliced, [-1, 0, undefined, 2, 3, 4]);
assert(spliced.hasOwnProperty(1));
assert(spliced.hasOwnProperty(3));
