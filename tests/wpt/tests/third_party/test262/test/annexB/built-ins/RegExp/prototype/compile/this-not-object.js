// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: Behavior when "this" value is not an Object
info: |
    1. Let O be the this value.
    2. If Type(O) is not Object or Type(O) is Object and O does not have a
       [[RegExpMatcher]] internal slot, then
       a. Throw a TypeError exception.
features: [Symbol]
---*/

var compile = RegExp.prototype.compile;
var symbol = Symbol('');

assert.sameValue(typeof compile, 'function');

assert.throws(TypeError, function() {
  compile.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  compile.call(null);
}, 'null');

assert.throws(TypeError, function() {
  compile.call(23);
}, 'number');

assert.throws(TypeError, function() {
  compile.call(true);
}, 'boolean');

assert.throws(TypeError, function() {
  compile.call('/string/');
}, 'string');

assert.throws(TypeError, function() {
  compile.call(symbol);
}, 'symbol');
