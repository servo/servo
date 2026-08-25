// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  Throws a TypeError if `this` is a Set object.
info: |
  Map.prototype.forEach ( callbackfn [ , thisArg ] )

  ...
  3. If M does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...
features: [Set]
---*/

assert.throws(TypeError, function() {
  Map.prototype.forEach.call(new Set(), function() {});
});

assert.throws(TypeError, function() {
  var m = new Map();
  m.forEach.call(new Set(), function() {});
});
