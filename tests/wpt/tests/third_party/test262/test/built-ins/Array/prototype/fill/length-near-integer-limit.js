// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.fill
description: >
  Elements are filled in an array-like object
  whose "length" property is near the integer limit.
info: |
  Array.prototype.fill ( value [ , start [ , end ] ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).
  [...]
  7. Repeat, while k < final
    a. Let Pk be ! ToString(k).
    b. Perform ? Set(O, Pk, value, true).
    [...]
---*/

var value = {};
var startIndex = Number.MAX_SAFE_INTEGER - 3;
var arrayLike = {
  length: Number.MAX_SAFE_INTEGER,
};

Array.prototype.fill.call(arrayLike, value, startIndex, startIndex + 3);

assert.sameValue(arrayLike[startIndex], value);
assert.sameValue(arrayLike[startIndex + 1], value);
assert.sameValue(arrayLike[startIndex + 2], value);
