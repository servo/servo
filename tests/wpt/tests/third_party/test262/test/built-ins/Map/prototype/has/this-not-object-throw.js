// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.has
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.has ( key )

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  Map.prototype.has.call(false, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.has.call(1, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.has.call('', 1);
});

assert.throws(TypeError, function() {
  Map.prototype.has.call(undefined, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.has.call(null, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.has.call(Symbol(), 1);
});

assert.throws(TypeError, function() {
  var map = new Map();
  map.has.call(false, 1);
});
