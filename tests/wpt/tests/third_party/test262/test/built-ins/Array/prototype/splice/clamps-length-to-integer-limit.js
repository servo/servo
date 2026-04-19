// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
  Length values exceeding 2^53-1 are clamped to 2^53-1.
info: |
  1. ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
  5. If the number of actual arguments is 0, then
    a. Let insertCount be 0.
    b. Let actualDeleteCount be 0.
  ...
  19. Perform ? Set(O, "length", len - actualDeleteCount + itemCount, true).
  ...
features: [exponentiation]
---*/

var arrayLike = {};

arrayLike.length = 2 ** 53 - 1;
Array.prototype.splice.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53 - 1");

arrayLike.length = 2 ** 53;
Array.prototype.splice.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53");

arrayLike.length = 2 ** 53 + 2;
Array.prototype.splice.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53 + 2");

arrayLike.length = Infinity;
Array.prototype.splice.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is Infinity");
