// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.1.5.2.1
description: >
  Throws a TypeError if `this` does not have all of the internal slots of a Map
  Iterator Instance.
info: |
  %MapIteratorPrototype%.next ( )

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have all of the internal slots of a Map Iterator Instance
  (23.1.5.3), throw a TypeError exception.
  ...
features: [Symbol.iterator]
---*/

var map = new Map([[1, 11], [2, 22]]);

var iterator = map[Symbol.iterator]();
assert.throws(TypeError, function() {
  iterator.next.call({});
});

iterator = map.entries();
assert.throws(TypeError, function() {
  iterator.next.call({});
});

iterator = map.keys();
assert.throws(TypeError, function() {
  iterator.next.call({});
});

iterator = map.values();
assert.throws(TypeError, function() {
  iterator.next.call({});
});

// does not throw an Error
iterator.next.call(map[Symbol.iterator]());
