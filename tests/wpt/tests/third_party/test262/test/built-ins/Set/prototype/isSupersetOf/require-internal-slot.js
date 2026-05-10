// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf RequireInternalSlot
info: |
    2. Perform ? RequireInternalSlot(O, [[SetData]])
features: [set-methods]
---*/

const isSupersetOf = Set.prototype.isSupersetOf;

assert.sameValue(typeof isSupersetOf, "function");

const realSet = new Set([]);

assert.throws(TypeError, () => isSupersetOf.call(undefined, realSet), "undefined");
assert.throws(TypeError, () => isSupersetOf.call(null, realSet), "null");
assert.throws(TypeError, () => isSupersetOf.call(true, realSet), "true");
assert.throws(TypeError, () => isSupersetOf.call("", realSet), "empty string");
assert.throws(TypeError, () => isSupersetOf.call(Symbol(), realSet), "symbol");
assert.throws(TypeError, () => isSupersetOf.call(1, realSet), "1");
assert.throws(TypeError, () => isSupersetOf.call(1n, realSet), "1n");
assert.throws(TypeError, () => isSupersetOf.call({}, realSet), "plain object");
assert.throws(TypeError, () => isSupersetOf.call([], realSet), "array");
assert.throws(TypeError, () => isSupersetOf.call(new Map(), realSet), "map");
assert.throws(TypeError, () => isSupersetOf.call(Set.prototype, realSet), "Set.prototype");
