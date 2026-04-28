// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Sune Eriksson Lianes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  1. Let M be the this value
  2. Perform ? RequireInternalSlot(M, [[MapData]])
  ...
features: [Symbol, arrow-function, upsert]
---*/
var m = new Map();

assert.throws(TypeError, function () {
    m.getOrInsertComputed.call(false, 1, () => 1);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed.call(1, 1, () => 1);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed.call("", 1, () => 1);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed.call(undefined, 1, () => 1);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed.call(null, 1, () => 1);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed.call(Symbol(), 1, () => 1);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed.call(1n, 1, () => 1);
});
