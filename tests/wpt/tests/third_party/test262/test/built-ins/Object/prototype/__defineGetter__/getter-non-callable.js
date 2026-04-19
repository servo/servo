// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
description: Behavior when getter is not callable
info: |
    [...]
    2. If IsCallable(getter) is false, throw a TypeError exception.
features: [Symbol, __getter__]
---*/

var subject = {};
var symbol = Symbol('');
var toStringCount = 0;
var key = {
  toString: function() {
    toStringCount += 1;
  }
};

assert.sameValue(typeof Object.prototype.__defineGetter__, 'function');

assert.throws(TypeError, function() {
  subject.__defineGetter__(key, '');
}, 'string');

assert.throws(TypeError, function() {
  subject.__defineGetter__(key, 23);
}, 'number');

assert.throws(TypeError, function() {
  subject.__defineGetter__(key, true);
}, 'boolean');

assert.throws(TypeError, function() {
  subject.__defineGetter__(key, symbol);
}, 'symbol');

assert.throws(TypeError, function() {
  subject.__defineGetter__(key, {});
}, 'object');

assert.sameValue(toStringCount, 0);
