// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: >
  Throws a TypeError if `this` object does not have a [[MapData]] internal slot.
info: |
  Map.prototype.set ( key , value )

  ...
  3. If M does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...
---*/

var m = new Map();

assert.throws(TypeError, function() {
  Map.prototype.set.call([], 1, 1);
});

assert.throws(TypeError, function() {
  m.set.call([], 1, 1);
});

assert.throws(TypeError, function() {
  Map.prototype.set.call({}, 1, 1);
});

assert.throws(TypeError, function() {
  m.set.call({}, 1, 1);
});
