// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
  A TypeError is thrown if the new length exceeds 2^53-1.
info: |
  1. ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
  7. Else,
    a. Let insertCount be the number of actual arguments minus 2.
    b. Let dc be ? ToInteger(deleteCount).
    c. Let actualDeleteCount be min(max(dc, 0), len - actualStart).
  8. If len+insertCount-actualDeleteCount > 2^53-1, throw a TypeError exception.
  ...
features: [exponentiation]
---*/

var arrayLike = {};

arrayLike.length = 2 ** 53 - 1;
assert.throws(TypeError, function() {
  Array.prototype.splice.call(arrayLike, 0, 0, null);
}, "Length is 2**53 - 1");

arrayLike.length = 2 ** 53;
assert.throws(TypeError, function() {
  Array.prototype.splice.call(arrayLike, 0, 0, null);
}, "Length is 2**53");

arrayLike.length = 2 ** 53 + 2;
assert.throws(TypeError, function() {
  Array.prototype.splice.call(arrayLike, 0, 0, null);
}, "Length is 2**53 + 2");

arrayLike.length = Infinity;
assert.throws(TypeError, function() {
  Array.prototype.splice.call(arrayLike, 0, 0, null);
}, "Length is Infinity");
