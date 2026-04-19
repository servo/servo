// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.negated
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const negated = Temporal.Duration.prototype.negated;

assert.sameValue(typeof negated, "function");

assert.throws(TypeError, () => negated.call(undefined), "undefined");
assert.throws(TypeError, () => negated.call(null), "null");
assert.throws(TypeError, () => negated.call(true), "true");
assert.throws(TypeError, () => negated.call(""), "empty string");
assert.throws(TypeError, () => negated.call(Symbol()), "symbol");
assert.throws(TypeError, () => negated.call(1), "1");
assert.throws(TypeError, () => negated.call({}), "plain object");
assert.throws(TypeError, () => negated.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => negated.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
