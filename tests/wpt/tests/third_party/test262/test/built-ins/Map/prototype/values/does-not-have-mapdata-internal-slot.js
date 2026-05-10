// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.values
description: >
  Throws a TypeError if `this` object does not have a [[MapData]] internal slot.
info: |
  Map.prototype.values ()

  1. Let M be the this value.
  2. Return CreateMapIterator(M, "value").

  23.1.5.1 CreateMapIterator Abstract Operation

  ...
  2. If map does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...

---*/

var m = new Map();

assert.throws(TypeError, function() {
  Map.prototype.values.call([]);
});

assert.throws(TypeError, function() {
  m.values.call([]);
});

assert.throws(TypeError, function() {
  Map.prototype.values.call({});
});

assert.throws(TypeError, function() {
  m.values.call({});
});
