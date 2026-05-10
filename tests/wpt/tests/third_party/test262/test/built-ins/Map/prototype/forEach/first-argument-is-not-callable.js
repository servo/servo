// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  Throws a TypeError if first argument is not callable.
info: |
  Map.prototype.forEach ( callbackfn [ , thisArg ] )

  4. If IsCallable(callbackfn) is false, throw a TypeError exception.
  ...
features: [Symbol]
---*/

var map = new Map();

assert.throws(TypeError, function() {
  map.forEach({});
});

assert.throws(TypeError, function() {
  map.forEach([]);
});

assert.throws(TypeError, function() {
  map.forEach(1);
});

assert.throws(TypeError, function() {
  map.forEach('');
});

assert.throws(TypeError, function() {
  map.forEach(null);
});

assert.throws(TypeError, function() {
  map.forEach(undefined);
});

assert.throws(TypeError, function() {
  map.forEach(Symbol());
});
