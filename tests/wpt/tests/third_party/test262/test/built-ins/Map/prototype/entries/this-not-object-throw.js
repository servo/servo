// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.entries
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.entries ( )

  ...
  2. Return CreateSetIterator(M, "key+value").

  23.1.5.1 CreateSetIterator Abstract Operation

  1. If Type(map) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  Map.prototype.entries.call(false);
});

assert.throws(TypeError, function() {
  Map.prototype.entries.call(1);
});

assert.throws(TypeError, function() {
  Map.prototype.entries.call('');
});

assert.throws(TypeError, function() {
  Map.prototype.entries.call(undefined);
});

assert.throws(TypeError, function() {
  Map.prototype.entries.call(null);
});

assert.throws(TypeError, function() {
  Map.prototype.entries.call(Symbol());
});

assert.throws(TypeError, function() {
  var map = new Map();
  map.entries.call(false);
});
