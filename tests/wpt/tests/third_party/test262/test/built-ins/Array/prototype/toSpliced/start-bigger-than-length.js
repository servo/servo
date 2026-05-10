// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced clamps the start argument to the this value length.
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  3. Let relativeStart be ? ToIntegerOrInfinity(start).
  4. If relativeStart is -∞, let actualStart be 0.
  5. Else if relativeStart < 0, let actualStart be max(len + relativeStart, 0).
  6. Else, let actualStart be min(relativeStart, len).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var result = [0, 1, 2, 3, 4].toSpliced(10, 1, 5, 6);
assert.compareArray(result, [0, 1, 2, 3, 4, 5, 6]);
