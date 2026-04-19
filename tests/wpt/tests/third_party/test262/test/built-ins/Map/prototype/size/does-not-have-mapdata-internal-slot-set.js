// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-map.prototype.size
description: >
  Throws a TypeError if `this` is a Set object.
info: |
  ...
  If M does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...
features: [Set]
---*/

var descriptor = Object.getOwnPropertyDescriptor(Map.prototype, 'size');

var map = new Map();

// Does not throw
descriptor.get.call(map);

assert.throws(TypeError, function() {
  descriptor.get.call(new Set());
});
