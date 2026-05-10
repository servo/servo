// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2025 Jonas Haukenes, Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.getorinsertcomputed
description: |
  Throws a TypeError if `callbackfn` is not callable.
info: |
  WeakMap.prototype.getOrInsertComputed ( key , callbackfn )

  ...
  3. If IsCallable(callbackfn) is false, throw a TypeError exception.
  ...
features: [WeakMap, Symbol, upsert]
---*/
var bar = {};
var m = new WeakMap();

assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, "");
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, true);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, undefined);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, null);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, {});
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, []);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, Symbol());
});

// Check that it also throws if the key is already present (thus it does not try to call the callback)
m.set(bar, "foo");
assert.throws(TypeError, function () {
    m.getOrInsertComputed(bar, 1);
});
