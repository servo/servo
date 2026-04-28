// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced treats a negative start as relative to the end.
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  3. Let relativeStart be ? ToIntegerOrInfinity(start).
  4. If relativeStart is -∞, let actualStart be 0.
  5. Else if relativeStart < 0, let actualStart be max(len + relativeStart, 0).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var result = [0, 1, 2, 3, 4].toSpliced(-3, 2);
assert.compareArray(result, [0, 1, 4]);
