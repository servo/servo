// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: >
    Behavior when thisArg is provided
    Array.prototype.flatMap ( mapperFunction [ , thisArg ] )
flags: [onlyStrict]
includes: [compareArray.js]
features: [Array.prototype.flatMap]
---*/

var a = {};
var actual;

actual = [1].flatMap(function() {
  return [this];
}, "TestString");
assert.compareArray(actual, ["TestString"], 'The value of actual is expected to be ["TestString"]');

actual = [1].flatMap(function() {
  return [this];
}, 1);
assert.compareArray(actual, [1], 'The value of actual is expected to be [1]');

actual = [1].flatMap(function() {
  return [this];
}, null);
assert.compareArray(actual, [null], 'The value of actual is expected to be [null]');

actual = [1].flatMap(function() {
  return [this];
}, true);
assert.compareArray(actual, [true], 'The value of actual is expected to be [true]');

actual = [1].flatMap(function() {
  return [this];
}, a);
assert.compareArray(actual, [a], 'The value of actual is expected to be [a]');

actual = [1].flatMap(function() {
  return [this];
}, void 0);
assert.compareArray(actual, [undefined], 'The value of actual is expected to be [undefined]');

