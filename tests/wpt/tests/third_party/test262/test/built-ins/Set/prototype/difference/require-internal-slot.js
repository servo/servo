// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.difference
description: Set.prototype.difference RequireInternalSlot
info: |
    2. Perform ? RequireInternalSlot(O, [[SetData]])
features: [set-methods]
---*/

const difference = Set.prototype.difference;

assert.sameValue(typeof difference, "function");

const realSet = new Set([]);

assert.throws(TypeError, () => difference.call(undefined, realSet), "undefined");
assert.throws(TypeError, () => difference.call(null, realSet), "null");
assert.throws(TypeError, () => difference.call(true, realSet), "true");
assert.throws(TypeError, () => difference.call("", realSet), "empty string");
assert.throws(TypeError, () => difference.call(Symbol(), realSet), "symbol");
assert.throws(TypeError, () => difference.call(1, realSet), "1");
assert.throws(TypeError, () => difference.call(1n, realSet), "1n");
assert.throws(TypeError, () => difference.call({}, realSet), "plain object");
assert.throws(TypeError, () => difference.call([], realSet), "array");
assert.throws(TypeError, () => difference.call(new Map(), realSet), "map");
assert.throws(TypeError, () => difference.call(Set.prototype, realSet), "Set.prototype");
