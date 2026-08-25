// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf RequireInternalSlot
info: |
    2. Perform ? RequireInternalSlot(O, [[SetData]])
features: [set-methods]
---*/

const isSubsetOf = Set.prototype.isSubsetOf;

assert.sameValue(typeof isSubsetOf, "function");

const realSet = new Set([]);

assert.throws(TypeError, () => isSubsetOf.call(undefined, realSet), "undefined");
assert.throws(TypeError, () => isSubsetOf.call(null, realSet), "null");
assert.throws(TypeError, () => isSubsetOf.call(true, realSet), "true");
assert.throws(TypeError, () => isSubsetOf.call("", realSet), "empty string");
assert.throws(TypeError, () => isSubsetOf.call(Symbol(), realSet), "symbol");
assert.throws(TypeError, () => isSubsetOf.call(1, realSet), "1");
assert.throws(TypeError, () => isSubsetOf.call(1n, realSet), "1n");
assert.throws(TypeError, () => isSubsetOf.call({}, realSet), "plain object");
assert.throws(TypeError, () => isSubsetOf.call([], realSet), "array");
assert.throws(TypeError, () => isSubsetOf.call(new Map(), realSet), "map");
assert.throws(TypeError, () => isSubsetOf.call(Set.prototype, realSet), "Set.prototype");
