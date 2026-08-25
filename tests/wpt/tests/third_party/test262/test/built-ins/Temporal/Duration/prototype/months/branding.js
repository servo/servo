// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.months
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const months = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "months").get;

assert.sameValue(typeof months, "function");

assert.throws(TypeError, () => months.call(undefined), "undefined");
assert.throws(TypeError, () => months.call(null), "null");
assert.throws(TypeError, () => months.call(true), "true");
assert.throws(TypeError, () => months.call(""), "empty string");
assert.throws(TypeError, () => months.call(Symbol()), "symbol");
assert.throws(TypeError, () => months.call(1), "1");
assert.throws(TypeError, () => months.call({}), "plain object");
assert.throws(TypeError, () => months.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => months.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
