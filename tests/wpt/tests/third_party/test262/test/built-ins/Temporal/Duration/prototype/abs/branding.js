// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.abs
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const abs = Temporal.Duration.prototype.abs;

assert.sameValue(typeof abs, "function");

assert.throws(TypeError, () => abs.call(undefined), "undefined");
assert.throws(TypeError, () => abs.call(null), "null");
assert.throws(TypeError, () => abs.call(true), "true");
assert.throws(TypeError, () => abs.call(""), "empty string");
assert.throws(TypeError, () => abs.call(Symbol()), "symbol");
assert.throws(TypeError, () => abs.call(1), "1");
assert.throws(TypeError, () => abs.call({}), "plain object");
assert.throws(TypeError, () => abs.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => abs.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
