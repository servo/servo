// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Sune Eriksson Lianes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsert
description: |
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.getOrInsert ( key , value )

  1. Let M be the this value
  2. Perform ? RequireInternalSlot(M, [[MapData]])
  ...
features: [Symbol, upsert]
---*/
var m = new Map();

assert.throws(TypeError, function () {
    m.getOrInsert.call(false, 1, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call(1, 1, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call("", 1, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call(undefined, 1, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call(null, 1, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call(Symbol(), 1, 1);
});

