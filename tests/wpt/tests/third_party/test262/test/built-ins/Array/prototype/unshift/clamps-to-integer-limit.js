// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.unshift
description: >
  Length values exceeding 2^53-1 are clamped to 2^53-1.
info: |
  1. ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. Let argCount be the number of actual arguments.
  4. If argCount > 0, then ...
  5. Perform ? Set(O, "length", len+argCount, true).
features: [exponentiation]
---*/

var arrayLike = {};

arrayLike.length = 2 ** 53 - 1;
Array.prototype.unshift.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53 - 1");

arrayLike.length = 2 ** 53;
Array.prototype.unshift.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53");

arrayLike.length = 2 ** 53 + 2;
Array.prototype.unshift.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is 2**53 + 2");

arrayLike.length = Infinity;
Array.prototype.unshift.call(arrayLike);
assert.sameValue(arrayLike.length, 2 ** 53 - 1, "Length is Infinity");
