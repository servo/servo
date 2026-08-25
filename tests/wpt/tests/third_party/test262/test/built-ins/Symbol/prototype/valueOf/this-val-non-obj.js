// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.prototype.valueof
description: Called on a value that is neither a Symbol nor an Object
info: |
  1. Let s be the this value.
  2. If Type(s) is Symbol, return s.
  3. If Type(s) is not Object, throw a TypeError exception.
features: [Symbol]
---*/

var valueOf = Symbol.prototype.valueOf;

assert.throws(TypeError, function() {
  valueOf.call(null);
}, 'null');

assert.throws(TypeError, function() {
  valueOf.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  valueOf.call(0);
}, 'number');

assert.throws(TypeError, function() {
  valueOf.call('');
}, 'string');
