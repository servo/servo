// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.get
description: >
  Throws a TypeError if `this` is a WeakMap object.
info: |
  Map.prototype.get ( key )

  ...
  3. If M does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...
features: [WeakMap]
---*/

assert.throws(TypeError, function() {
  Map.prototype.get.call(new WeakMap(), 1);
});

assert.throws(TypeError, function() {
  var m = new Map();
  m.get.call(new WeakMap(), 1);
});
