// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: Array.prototype.toSpliced returns a copy of the array if called with zero arguments
info: |
  22.1.3.25 Array.prototype.toSpliced (start, deleteCount , ...items )

  ...
  3. Let relativeStart be ? ToIntegerOrInfinity(start).
  ...
  6. Else, let actualStart be min(relativeStart, len).
  ...
  8. If start is not present, then
    a. Let actualDeleteCount be 0.
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

let arr = ["first", "second", "third"];
let result = arr.toSpliced();

assert.compareArray(result, arr);
assert.notSameValue(result, arr);

