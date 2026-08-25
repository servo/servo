// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.get
description: >
  Throws a TypeError if `this` is a Set object.
info: |
  WeakMap.prototype.get ( key )

  ...
  3. If M does not have a [[WeakMapData]] internal slot, throw a TypeError
  exception.
  ...
features: [Set]
---*/

assert.throws(TypeError, function() {
  WeakMap.prototype.get.call(new Set(), 1);
});

assert.throws(TypeError, function() {
  var map = new WeakMap();
  map.get.call(new Set(), 1);
});
