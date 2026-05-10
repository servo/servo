// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.indexof
description: >
  Returns -1 if length is 0.
info: |
  Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).
  3. If len is 0, return -1.
---*/

var fromIndex = {
  valueOf: function() {
    throw new Test262Error("Length should be checked before ToInteger(fromIndex).");
  },
};

assert.sameValue([].indexOf(1), -1);
assert.sameValue([].indexOf(2, fromIndex), -1);
