// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Array.prototype.with throws if the (negative) index is smaller than -length.
info: |
  Array.prototype.with ( index, value )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  3. Let relativeIndex be ? ToIntegerOrInfinity(index).
  4. If index >= 0, let actualIndex be relativeIndex.
  5. Else, let actualIndex be len + relativeIndex.
  6. If actualIndex >= len or actualIndex < 0, throw a *RangeError* exception.
  ...
features: [change-array-by-copy, exponentiation]
---*/

[0, 1, 2].with(-3, 7);

assert.throws(RangeError, function() {
  [0, 1, 2].with(-4, 7);
});

assert.throws(RangeError, function() {
  [0, 1, 2].with(-10, 7);
});

assert.throws(RangeError, function() {
  [0, 1, 2].with(-(2 ** 53) - 2, 7);
});

assert.throws(RangeError, function() {
  [0, 1, 2].with(-Infinity, 7);
});
