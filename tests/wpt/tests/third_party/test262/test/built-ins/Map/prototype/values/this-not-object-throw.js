// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.values
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.values ()

  ...
  2. Return CreateMapIterator(M, "values").

  23.1.5.1 CreateMapIterator Abstract Operation

  1. If Type(map) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  Map.prototype.values.call(false);
});

assert.throws(TypeError, function() {
  Map.prototype.values.call(1);
});

assert.throws(TypeError, function() {
  Map.prototype.values.call('');
});

assert.throws(TypeError, function() {
  Map.prototype.values.call(undefined);
});

assert.throws(TypeError, function() {
  Map.prototype.values.call(null);
});

assert.throws(TypeError, function() {
  Map.prototype.values.call(Symbol());
});

assert.throws(TypeError, function() {
  var map = new Map();
  map.values.call(false);
});
