// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.has
description: >
  Throws a TypeError if `this` is a Set object.
info: |
  Map.prototype.has ( key )

  ...
  3. If M does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...
features: [Set]
---*/

assert.throws(TypeError, function() {
  Map.prototype.has.call(new Set(), 1);
});

assert.throws(TypeError, function() {
  var m = new Map();
  m.has.call(new Set(), 1);
});
