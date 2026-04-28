// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getseconds
description: Behavior when "this" value is not an Object
info: |
  1. Let t be ? thisTimeValue(this value).

  The abstract operation thisTimeValue(value) performs the following steps:

  1. If Type(value) is Object and value has a [[DateValue]] internal slot, then
     a. Return value.[[DateValue]].
  2. Throw a TypeError exception.
features: [Symbol]
---*/

var getSeconds = Date.prototype.getSeconds;
var symbol = Symbol();

assert.sameValue(typeof getSeconds, 'function');

assert.throws(TypeError, function() {
  getSeconds.call(0);
}, 'number');

assert.throws(TypeError, function() {
  getSeconds.call(true);
}, 'boolean');

assert.throws(TypeError, function() {
  getSeconds.call(null);
}, 'null');

assert.throws(TypeError, function() {
  getSeconds.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  getSeconds.call('');
}, 'string');

assert.throws(TypeError, function() {
  getSeconds.call(symbol);
}, 'symbol');
