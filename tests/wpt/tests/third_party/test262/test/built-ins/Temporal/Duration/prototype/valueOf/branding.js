// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.valueof
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const valueOf = Temporal.Duration.prototype.valueOf;

assert.sameValue(typeof valueOf, "function");

assert.throws(TypeError, () => valueOf.call(undefined), "undefined");
assert.throws(TypeError, () => valueOf.call(null), "null");
assert.throws(TypeError, () => valueOf.call(true), "true");
assert.throws(TypeError, () => valueOf.call(""), "empty string");
assert.throws(TypeError, () => valueOf.call(Symbol()), "symbol");
assert.throws(TypeError, () => valueOf.call(1), "1");
assert.throws(TypeError, () => valueOf.call({}), "plain object");
assert.throws(TypeError, () => valueOf.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => valueOf.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
