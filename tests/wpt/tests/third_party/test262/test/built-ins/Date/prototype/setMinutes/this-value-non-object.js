// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setminutes
description: Behavior when "this" value is not an Object
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).

  The abstract operation thisTimeValue(value) performs the following steps:

  1. If Type(value) is Object and value has a [[DateValue]] internal slot, then
     a. Return value.[[DateValue]].
  2. Throw a TypeError exception.
features: [Symbol]
---*/

var setMinutes = Date.prototype.setMinutes;
var callCount = 0;
var arg = {
  valueOf: function() {
    callCount += 1;
    return 1;
  }
};
var symbol = Symbol();

assert.sameValue(typeof setMinutes, 'function');

assert.throws(TypeError, function() {
  setMinutes.call(0, arg);
}, 'number');

assert.throws(TypeError, function() {
  setMinutes.call(true, arg);
}, 'boolean');

assert.throws(TypeError, function() {
  setMinutes.call(null, arg);
}, 'null');

assert.throws(TypeError, function() {
  setMinutes.call(undefined, arg);
}, 'undefined');

assert.throws(TypeError, function() {
  setMinutes.call('', arg);
}, 'string');

assert.throws(TypeError, function() {
  setMinutes.call(symbol, arg);
}, 'symbol');

assert.sameValue(callCount, 0, 'validation precedes input coercion');
