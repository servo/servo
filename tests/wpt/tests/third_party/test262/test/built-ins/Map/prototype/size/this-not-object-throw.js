// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-map.prototype.size
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  get Map.prototype.size

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
  3. If M does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...

features: [Symbol]
---*/

var descriptor = Object.getOwnPropertyDescriptor(Map.prototype, 'size');

assert.throws(TypeError, function() {
  descriptor.get.call(1);
});

assert.throws(TypeError, function() {
  descriptor.get.call(false);
});

assert.throws(TypeError, function() {
  descriptor.get.call(1);
});

assert.throws(TypeError, function() {
  descriptor.get.call('');
});

assert.throws(TypeError, function() {
  descriptor.get.call(undefined);
});

assert.throws(TypeError, function() {
  descriptor.get.call(null);
});

assert.throws(TypeError, function() {
  descriptor.get.call(Symbol());
});
