// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.2.5.2.1
description: >
  Throws a TypeError if `this` does not have all of the internal slots of a Set
  Iterator Instance.
info: |
  %SetIteratorPrototype%.next ( )

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have all of the internal slots of a Set Iterator Instance
  (23.2.5.3), throw a TypeError exception.
  ...
features: [Symbol.iterator]
---*/

var set = new Set([1, 2]);

var iterator = set[Symbol.iterator]();
assert.throws(TypeError, function() {
  iterator.next.call({});
});

iterator = set.entries();
assert.throws(TypeError, function() {
  iterator.next.call({});
});

iterator = set.keys();
assert.throws(TypeError, function() {
  iterator.next.call({});
});

iterator = set.values();
assert.throws(TypeError, function() {
  iterator.next.call({});
});

// does not throw an Error
iterator.next.call(set[Symbol.iterator]());
