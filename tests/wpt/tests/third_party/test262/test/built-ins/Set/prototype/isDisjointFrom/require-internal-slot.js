// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.isdisjointfrom
description: Set.prototype.isDisjointFrom RequireInternalSlot
info: |
    2. Perform ? RequireInternalSlot(O, [[SetData]])
features: [set-methods]
---*/

const isDisjointFrom = Set.prototype.isDisjointFrom;

assert.sameValue(typeof isDisjointFrom, "function");

const realSet = new Set([]);

assert.throws(TypeError, () => isDisjointFrom.call(undefined, realSet), "undefined");
assert.throws(TypeError, () => isDisjointFrom.call(null, realSet), "null");
assert.throws(TypeError, () => isDisjointFrom.call(true, realSet), "true");
assert.throws(TypeError, () => isDisjointFrom.call("", realSet), "empty string");
assert.throws(TypeError, () => isDisjointFrom.call(Symbol(), realSet), "symbol");
assert.throws(TypeError, () => isDisjointFrom.call(1, realSet), "1");
assert.throws(TypeError, () => isDisjointFrom.call(1n, realSet), "1n");
assert.throws(TypeError, () => isDisjointFrom.call({}, realSet), "plain object");
assert.throws(TypeError, () => isDisjointFrom.call([], realSet), "array");
assert.throws(TypeError, () => isDisjointFrom.call(new Map(), realSet), "map");
assert.throws(TypeError, () => isDisjointFrom.call(Set.prototype, realSet), "Set.prototype");
