// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The `this` value must be an object
es6id: 21.2.5.9
info: |
    1. Let rx be the this value.
    2. If Type(rx) is not Object, throw a TypeError exception.
features: [Symbol.search]
---*/

assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call();
});

assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(undefined);
});

assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(null);
});

assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(true);
});

assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call('string');
});

assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(Symbol.search);
});

assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(86);
});
