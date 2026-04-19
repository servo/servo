// Copyright (C) 2023 Anthony Frehner. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.union
description: Set.prototype.union RequireInternalSlot
info: |
    2. Perform ? RequireInternalSlot(O, [[SetData]])
features: [set-methods]
---*/

const union = Set.prototype.union;

assert.sameValue(typeof union, "function");

const realSet = new Set([]);

assert.throws(TypeError, () => union.call(undefined, realSet), "undefined");
assert.throws(TypeError, () => union.call(null, realSet), "null");
assert.throws(TypeError, () => union.call(true, realSet), "true");
assert.throws(TypeError, () => union.call("", realSet), "empty string");
assert.throws(TypeError, () => union.call(Symbol(), realSet), "symbol");
assert.throws(TypeError, () => union.call(1, realSet), "1");
assert.throws(TypeError, () => union.call(1n, realSet), "1n");
assert.throws(TypeError, () => union.call({}, realSet), "plain object");
assert.throws(TypeError, () => union.call([], realSet), "array");
assert.throws(TypeError, () => union.call(new Map(), realSet), "map");
assert.throws(TypeError, () => union.call(Set.prototype, realSet), "Set.prototype");
