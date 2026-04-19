// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.symmetricdifference
description: Set.prototype.symmetricDifference RequireInternalSlot
info: |
    2. Perform ? RequireInternalSlot(O, [[SetData]])
features: [set-methods]
---*/

const symmetricDifference = Set.prototype.symmetricDifference;

assert.sameValue(typeof symmetricDifference, "function");

const realSet = new Set([]);

assert.throws(TypeError, () => symmetricDifference.call(undefined, realSet), "undefined");
assert.throws(TypeError, () => symmetricDifference.call(null, realSet), "null");
assert.throws(TypeError, () => symmetricDifference.call(true, realSet), "true");
assert.throws(TypeError, () => symmetricDifference.call("", realSet), "empty string");
assert.throws(TypeError, () => symmetricDifference.call(Symbol(), realSet), "symbol");
assert.throws(TypeError, () => symmetricDifference.call(1, realSet), "1");
assert.throws(TypeError, () => symmetricDifference.call(1n, realSet), "1n");
assert.throws(TypeError, () => symmetricDifference.call({}, realSet), "plain object");
assert.throws(TypeError, () => symmetricDifference.call([], realSet), "array");
assert.throws(TypeError, () => symmetricDifference.call(new Map(), realSet), "map");
assert.throws(TypeError, () => symmetricDifference.call(Set.prototype, realSet), "Set.prototype");
