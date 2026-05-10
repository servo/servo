// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.push
description: >
  A TypeError is thrown if the new length exceeds 2^53-1.
info: |
  1. ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. Let items be a List whose elements are, in left to right order, the arguments
     that were passed to this function invocation.
  4. Let argCount be the number of elements in items.
  5. If len + argCount > 2^53-1, throw a TypeError exception.
  ...
features: [exponentiation]
---*/

var arrayLike = {};

arrayLike.length = 2 ** 53 - 1;
assert.throws(TypeError, function() {
  Array.prototype.push.call(arrayLike, null);
}, "Length is 2**53 - 1");

arrayLike.length = 2 ** 53;
assert.throws(TypeError, function() {
  Array.prototype.push.call(arrayLike, null);
}, "Length is 2**53");

arrayLike.length = 2 ** 53 + 2;
assert.throws(TypeError, function() {
  Array.prototype.push.call(arrayLike, null);
}, "Length is 2**53 + 2");

arrayLike.length = Infinity;
assert.throws(TypeError, function() {
  Array.prototype.push.call(arrayLike, null);
}, "Length is Infinity");
