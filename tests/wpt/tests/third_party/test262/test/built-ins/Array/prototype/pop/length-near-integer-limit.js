// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.pop
description: >
  A value is removed from an array-like object whose length property is near the integer limit.
info: |
  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
  4. Else len > 0,
    a. Let newLen be len-1.
    b. Let index be ! ToString(newLen).
    c. Let element be ? Get(O, index).
    d. Perform ? DeletePropertyOrThrow(O, index).
    e. Perform ? Set(O, "length", newLen, true).
    f. Return element.
features: [exponentiation]
---*/

var arrayLike = {
  "9007199254740989": "9007199254740989",
  "9007199254740990": "9007199254740990",
  "9007199254740991": "9007199254740991",
  length: 2 ** 53 - 1
};

var value = Array.prototype.pop.call(arrayLike);

assert.sameValue(value, "9007199254740990",
  "arrayLike['9007199254740990'] is returned from pop()");

assert.sameValue(arrayLike.length, 2 ** 53 - 2,
  "New arrayLike.length is 2**53 - 2");

assert.sameValue(arrayLike["9007199254740989"], "9007199254740989",
  "arrayLike['9007199254740989'] is unchanged");

assert.sameValue("9007199254740990" in arrayLike, false,
  "arrayLike['9007199254740990'] is removed");

assert.sameValue(arrayLike["9007199254740991"], "9007199254740991",
  "arrayLike['9007199254740991'] is unchanged");
