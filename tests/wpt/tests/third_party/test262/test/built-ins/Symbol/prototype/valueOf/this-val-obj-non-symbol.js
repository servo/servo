// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.prototype.valueof
description: Called on an Object value that is not a Symbol object
info: |
  1. Let s be the this value.
  2. If Type(s) is Symbol, return s.
  3. If Type(s) is not Object, throw a TypeError exception.
  4. If s does not have a [[SymbolData]] internal slot, throw a TypeError exception.
features: [Symbol]
---*/

var valueOf = Symbol.prototype.valueOf;

assert.throws(TypeError, function() {
  valueOf.call({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  valueOf.call([]);
}, 'array exotic object');

assert.throws(TypeError, function() {
  valueOf.call(arguments);
}, 'arguments exotic object');
