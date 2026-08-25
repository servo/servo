// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.unshift
description: >
  A TypeError is thrown if the new length exceeds 2^53-1.
info: |
  1. ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. Let argCount be the number of actual arguments.
  4. If argCount > 0, then
    a. If len+argCount > 2^53-1, throw a TypeError exception.
    b. ...
features: [exponentiation]
---*/

var arrayLike = {};

arrayLike.length = 2 ** 53 - 1;
assert.throws(TypeError, function() {
  Array.prototype.unshift.call(arrayLike, null);
}, "Length is 2**53 - 1");

arrayLike.length = 2 ** 53;
assert.throws(TypeError, function() {
  Array.prototype.unshift.call(arrayLike, null);
}, "Length is 2**53");

arrayLike.length = 2 ** 53 + 2;
assert.throws(TypeError, function() {
  Array.prototype.unshift.call(arrayLike, null);
}, "Length is 2**53 + 2");

arrayLike.length = Infinity;
assert.throws(TypeError, function() {
  Array.prototype.unshift.call(arrayLike, null);
}, "Length is Infinity");
