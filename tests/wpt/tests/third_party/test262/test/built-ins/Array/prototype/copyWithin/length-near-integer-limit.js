// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.copywithin
description: >
  Elements are copied and deleted in an array-like object
  whose "length" property is near the integer limit.
info: |
  Array.prototype.copyWithin ( target, start [ , end ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).
  [...]
  9. Let count be min(final - from, len - to).
  [...]
  12. Repeat, while count > 0
    [...]
    d. If fromPresent is true, then
      i. Let fromVal be ? Get(O, fromKey).
      ii. Perform ? Set(O, toKey, fromVal, true).
    e. Else,
      i. Assert: fromPresent is false.
      ii. Perform ? DeletePropertyOrThrow(O, toKey).
    [...]
---*/

var startIndex = Number.MAX_SAFE_INTEGER - 3;
var arrayLike = {
  0: 0,
  1: 1,
  2: 2,
  length: Number.MAX_SAFE_INTEGER,
};

arrayLike[startIndex] = -3;
arrayLike[startIndex + 2] = -1;

Array.prototype.copyWithin.call(arrayLike, 0, startIndex, startIndex + 3);

assert.sameValue(arrayLike[0], -3);
assert.sameValue(1 in arrayLike, false);
assert.sameValue(arrayLike[2], -1);
