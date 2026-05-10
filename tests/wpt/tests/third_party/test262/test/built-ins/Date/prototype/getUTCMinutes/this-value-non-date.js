// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getutcminutes
description: >
  Behavior when "this" value is an Object without a [[DateValue]] internal slot
info: |
  1. Let t be ? thisTimeValue(this value).

  The abstract operation thisTimeValue(value) performs the following steps:

  1. If Type(value) is Object and value has a [[DateValue]] internal slot, then
     a. Return value.[[DateValue]].
  2. Throw a TypeError exception.
---*/

var getUTCMinutes = Date.prototype.getUTCMinutes;
var args = (function() {
  return arguments;
}());

assert.sameValue(typeof getUTCMinutes, 'function');

assert.throws(TypeError, function() {
  getUTCMinutes.call({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  getUTCMinutes.call([]);
}, 'array exotic object');

assert.throws(TypeError, function() {
  getUTCMinutes.call(args);
}, 'arguments exotic object');
