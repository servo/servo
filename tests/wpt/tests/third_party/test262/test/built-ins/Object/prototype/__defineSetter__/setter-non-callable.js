// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
description: Behavior when getter is not callable
info: |
    [...]
    2. If IsCallable(setter) is false, throw a TypeError exception.
features: [Symbol, __setter__]
---*/

var subject = {};
var symbol = Symbol('');
var toStringCount = 0;
var key = {
  toString: function() {
    toStringCount += 1;
  }
};

assert.sameValue(typeof Object.prototype.__defineSetter__, 'function');

assert.throws(TypeError, function() {
  subject.__defineSetter__(key, '');
}, 'string');

assert.throws(TypeError, function() {
  subject.__defineSetter__(key, 23);
}, 'number');

assert.throws(TypeError, function() {
  subject.__defineSetter__(key, true);
}, 'boolean');

assert.throws(TypeError, function() {
  subject.__defineSetter__(key, symbol);
}, 'symbol');

assert.throws(TypeError, function() {
  subject.__defineSetter__(key, {});
}, 'object');

assert.sameValue(toStringCount, 0);
