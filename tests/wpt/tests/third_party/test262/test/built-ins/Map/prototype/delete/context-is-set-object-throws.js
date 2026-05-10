// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.delete
description: >
  Throws a TypeError if `this` is a Set object.
info: |
  Map.prototype.delete ( key )

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
  3. If M does not have a [[MapData]] internal slot, throw a TypeError
  exception.
  ...
features: [Set]
---*/

assert.throws(TypeError, function() {
  Map.prototype.delete.call(new Set(), 'attr');
});
