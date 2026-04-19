// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.1.5.2.1
description: >
  Throws a TypeError if `this` value is not an Object.
info: |
  Using Map.prototype[Symbol.iterator]()

  %MapIteratorPrototype%.next ( )

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  ...
features:
  - Symbol
  - Symbol.iterator
---*/

var map = new Map([[1, 11], [2, 22]]);
var iterator = map[Symbol.iterator]();

assert.throws(TypeError, function() {
  iterator.next.call(false);
});

assert.throws(TypeError, function() {
  iterator.next.call(1);
});

assert.throws(TypeError, function() {
  iterator.next.call('');
});

assert.throws(TypeError, function() {
  iterator.next.call(undefined);
});

assert.throws(TypeError, function() {
  iterator.next.call(null);
});

assert.throws(TypeError, function() {
  iterator.next.call(Symbol());
});

// does not throw an Error
iterator.next.call(map[Symbol.iterator]());
