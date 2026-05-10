// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.pop
description: >
  Length values exceeding 2^53-1 are clamped to 2^53-1.
info: |
  1. ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
  4. Else len > 0,
  a. Let newLen be len-1.
  ...
  e. Perform ? Set(O, "length", newLen, true).
  ...
features: [exponentiation]
---*/

var arrayLike = {};

arrayLike.length = 2 ** 53 - 1;
Array.prototype.pop.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 2, "Length is 2**53 - 1");

arrayLike.length = 2 ** 53;
Array.prototype.pop.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 2, "Length is 2**53");

arrayLike.length = 2 ** 53 + 2;
Array.prototype.pop.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 2, "Length is 2**53 + 2");

arrayLike.length = Infinity;
Array.prototype.pop.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 2, "Length is Infinity");
