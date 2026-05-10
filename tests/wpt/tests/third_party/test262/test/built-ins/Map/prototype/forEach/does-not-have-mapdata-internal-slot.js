// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  Throws a TypeError if `this` object does not have a [[MapData]] internal slot.
info: |
  Map.prototype.forEach ( callbackfn [ , thisArg ] )

  ...
  3. If M does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...
---*/

var m = new Map();

assert.throws(TypeError, function() {
  Map.prototype.forEach.call([], function() {});
});

assert.throws(TypeError, function() {
  m.forEach.call([], function() {});
});

assert.throws(TypeError, function() {
  Map.prototype.forEach.call({}, function() {});
});

assert.throws(TypeError, function() {
  m.forEach.call({}, function() {});
});
