// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2025 Jonas Haukenes, Sune Eriksson Lianes. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getOrInsert
description: |
  Throws a TypeError if `this` is not an Object.
info: |
  WeakMap.prototype.getOrInsert ( key , value )

  1. Let M be the this value
  2. Perform ? RequireInternalSlot(M, [[WeakMapData]])
  ...
features: [Symbol, upsert]
---*/
var m = new WeakMap();

assert.throws(TypeError, function () {
    m.getOrInsert.call(false, {}, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call(1, {}, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call("", {}, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call(undefined, {}, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call(null, {}, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call(Symbol(), {}, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsert.call('', {}, 1);
});

