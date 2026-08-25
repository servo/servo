// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: deleteCount is clamped between zero and len - actualStart
info: |
  22.1.3.25 Array.prototype.toSpliced (start, deleteCount , ...items )

  ...
  10. Else,
    a. Let dc be ? ToIntegerOrInfinity(deleteCount).
    b. Let actualDeleteCount be the result of clamping dc between 0 and len - actualStart.
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3, 4, 5].toSpliced(2, -1),
  [0, 1, 2, 3, 4, 5]
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].toSpliced(-4, -1),
  [0, 1, 2, 3, 4, 5]
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].toSpliced(2, 6),
  [0, 1]
);

assert.compareArray(
  [0, 1, 2, 3, 4, 5].toSpliced(-4, 6),
  [0, 1]
);
