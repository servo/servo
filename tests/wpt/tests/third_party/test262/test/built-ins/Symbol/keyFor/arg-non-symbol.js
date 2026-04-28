// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.keyfor
description: Called with a non-symbol argument
info: |
    1. If Type(sym) is not Symbol, throw a TypeError exception.
features: [Symbol]
---*/

assert.sameValue(typeof Symbol.keyFor, 'function');

assert.throws(TypeError, function() {
  Symbol.keyFor(null);
}, 'null');

assert.throws(TypeError, function() {
  Symbol.keyFor(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  Symbol.keyFor('1');
}, 'number');

assert.throws(TypeError, function() {
  Symbol.keyFor('');
}, 'string');

assert.throws(TypeError, function() {
  Symbol.keyFor({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  Symbol.keyFor([]);
}, 'array exotic object');

assert.throws(TypeError, function() {
  Symbol.keyFor(arguments);
}, 'arguments exotic object');

var subject = Object(Symbol('s'));

assert.throws(TypeError, function() {
  Symbol.keyFor(subject);
}, 'symbol object');
