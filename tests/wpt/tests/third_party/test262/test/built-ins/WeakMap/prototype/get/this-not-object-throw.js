// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.get
description: >
  Throws a TypeError if `this` value is not an Object.
info: |
  WeakMap.prototype.get ( key )

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  WeakMap.prototype.get.call(false, {});
});

assert.throws(TypeError, function() {
  WeakMap.prototype.get.call(1, {});
});

assert.throws(TypeError, function() {
  WeakMap.prototype.get.call('', {});
});

assert.throws(TypeError, function() {
  WeakMap.prototype.get.call(undefined, {});
});

assert.throws(TypeError, function() {
  WeakMap.prototype.get.call(null, {});
});

assert.throws(TypeError, function() {
  WeakMap.prototype.get.call(Symbol(), {});
});

assert.throws(TypeError, function() {
  var map = new WeakMap();
  map.get.call(false, {});
});
