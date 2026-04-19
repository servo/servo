// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.3.4
description: Behavior when `this` value is neither a Symbol nor an Object
info: |
    1. Let s be the this value.
    2. If Type(s) is Symbol, return s.
    3. If Type(s) is not Object, throw a TypeError exception.
features: [Symbol.toPrimitive]
---*/

assert.sameValue(typeof Symbol.prototype[Symbol.toPrimitive], 'function');

assert.throws(TypeError, function() {
  Symbol.prototype[Symbol.toPrimitive].call(undefined);
});

assert.throws(TypeError, function() {
  Symbol.prototype[Symbol.toPrimitive].call(null);
});

assert.throws(TypeError, function() {
  Symbol.prototype[Symbol.toPrimitive].call(86);
});

assert.throws(TypeError, function() {
  Symbol.prototype[Symbol.toPrimitive].call('');
});

assert.throws(TypeError, function() {
  Symbol.prototype[Symbol.toPrimitive].call(true);
});
