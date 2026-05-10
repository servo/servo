// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2024 Mathias Ness. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.getorinsertcomputed
description: |
  Throws a TypeError if `callbackfn` is not callable.
info: |
  Map.prototype.getOrInsertComputed ( key , callbackfn )

  ...
  3. If IsCallable(callbackfn) is false, throw a TypeError exception.
  ...
features: [Symbol, upsert]
---*/
var m = new Map();

assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, 1);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, "");
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, true);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, undefined);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, null);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, {});
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, []);
});

assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, Symbol());
});

// Check that it also throws if the key is already present (thus it does not try to call the callback)
m.set(1, "foo");
assert.throws(TypeError, function () {
    m.getOrInsertComputed(1, 1);
});
