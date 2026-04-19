// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.set ( key , value )

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  Map.prototype.set.call(false, 1, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.set.call(1, 1, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.set.call('', 1, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.set.call(undefined, 1, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.set.call(null, 1, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.set.call(Symbol(), 1, 1);
});

assert.throws(TypeError, function() {
  var map = new Map();
  map.set.call(false, 1, 1);
});
