// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Sune Eriksson Lianes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Throws a TypeError if `this` is a WeakMap object.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  ...
  1. Let M be the this value.
  2. Perform ? RequireInternalSlot(M, [[MapData]]).
  ...
features: [WeakMap, arrow-function, upsert]
---*/
var weakmap = new WeakMap();
assert.throws(TypeError, function() {
  Map.prototype.getOrInsertComputed.call(weakmap, 1, () => 1);
});

var map = new Map();
assert.throws(TypeError, function() {
  map.getOrInsertComputed.call(weakmap, 1, () => 1);
});

