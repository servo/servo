// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.forEach ( callbackfn [ , thisArg ] )

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  Map.prototype.forEach.call(false, function() {});
});

assert.throws(TypeError, function() {
  Map.prototype.forEach.call(1, function() {});
});

assert.throws(TypeError, function() {
  Map.prototype.forEach.call('', function() {});
});

assert.throws(TypeError, function() {
  Map.prototype.forEach.call(undefined, function() {});
});

assert.throws(TypeError, function() {
  Map.prototype.forEach.call(null, function() {});
});

assert.throws(TypeError, function() {
  Map.prototype.forEach.call(Symbol(), function() {});
});

assert.throws(TypeError, function() {
  var map = new Map();
  map.forEach.call(false, function() {});
});
