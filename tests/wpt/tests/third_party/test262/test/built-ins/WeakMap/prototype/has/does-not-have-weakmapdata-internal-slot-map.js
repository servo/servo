// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.has
description: >
  Throws TypeError if `this` doesn't have a [[WeakMapData]] internal slot.
info: |
  WeakMap.prototype.has ( value )

  ...
  3. If M does not have a [[WeakMapData]] internal slot, throw a TypeError
  exception.
  ...
features: [Map]
---*/

assert.throws(TypeError, function() {
  WeakMap.prototype.has.call(new Map(), {}, 1);
});

assert.throws(TypeError, function() {
  var map = new WeakMap();
  map.has.call(new Map(), {}, 1);
});
