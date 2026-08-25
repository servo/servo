// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Array.prototype.with adds length to index if it's negative.
info: |
  Array.prototype.with ( index, value )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  3. Let relativeIndex be ? ToIntegerOrInfinity(index).
  4. If index >= 0, let actualIndex be relativeIndex.
  5. Else, let actualIndex be len + relativeIndex.
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [0, 1, 2];

assert.compareArray(arr.with(-1, 4), [0, 1, 4]);
assert.compareArray(arr.with(-3, 4), [4, 1, 2]);

// -0 is not < 0
assert.compareArray(arr.with(-0, 4), [4, 1, 2]);
