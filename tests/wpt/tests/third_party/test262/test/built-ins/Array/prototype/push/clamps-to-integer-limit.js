// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.push
description: >
  Length values exceeding 2^53-1 are clamped to 2^53-1.
info: |
  1. ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. Let items be a List whose elements are, in left to right order, the arguments
     that were passed to this function invocation.
  4. Let argCount be the number of elements in items.
  ...
  7. Perform ? Set(O, "length", len, true).
  ...
features: [exponentiation]
---*/

var arrayLike = {};

arrayLike.length = 2 ** 53 - 1;
Array.prototype.push.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53 - 1");

arrayLike.length = 2 ** 53;
Array.prototype.push.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53");

arrayLike.length = 2 ** 53 + 2;
Array.prototype.push.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53 + 2");

arrayLike.length = Infinity;
Array.prototype.push.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is Infinity");
