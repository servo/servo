// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Sune Eriksson Lianes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsert
description: |
  Throws a TypeError if `this` is a Set Object
info: |
  Map.prototype.getOrInsert ( key , value )

  ...
  1. Let M be the this value.
  2. Perform ? RequireInternalSlot(M, [[MapData]])
  ...
features: [Set, upsert]
---*/
assert.throws(TypeError, function () {
  Map.prototype.getOrInsert.call(new Set(), 1, 1);
});

assert.throws(TypeError, function () {
  var map = new Map();
  map.getOrInsert.call(new Set(), 1, 1);
});

