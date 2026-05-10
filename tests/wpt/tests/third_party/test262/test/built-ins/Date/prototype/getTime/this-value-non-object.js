// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.gettime
description: Behavior when "this" value is not an Object
info: |
  1. Return ? thisTimeValue(this value). 

  The abstract operation thisTimeValue(value) performs the following steps:

  1. If Type(value) is Object and value has a [[DateValue]] internal slot, then
     a. Return value.[[DateValue]].
  2. Throw a TypeError exception.
features: [Symbol]
---*/

var getTime = Date.prototype.getTime;
var symbol = Symbol();

assert.sameValue(typeof getTime, 'function');

assert.throws(TypeError, function() {
  getTime.call(0);
}, 'number');

assert.throws(TypeError, function() {
  getTime.call(true);
}, 'boolean');

assert.throws(TypeError, function() {
  getTime.call(null);
}, 'null');

assert.throws(TypeError, function() {
  getTime.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  getTime.call('');
}, 'string');

assert.throws(TypeError, function() {
  getTime.call(symbol);
}, 'symbol');
