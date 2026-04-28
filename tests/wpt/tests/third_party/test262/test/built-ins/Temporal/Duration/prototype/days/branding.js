// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.days
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const days = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "days").get;

assert.sameValue(typeof days, "function");

assert.throws(TypeError, () => days.call(undefined), "undefined");
assert.throws(TypeError, () => days.call(null), "null");
assert.throws(TypeError, () => days.call(true), "true");
assert.throws(TypeError, () => days.call(""), "empty string");
assert.throws(TypeError, () => days.call(Symbol()), "symbol");
assert.throws(TypeError, () => days.call(1), "1");
assert.throws(TypeError, () => days.call({}), "plain object");
assert.throws(TypeError, () => days.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => days.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
