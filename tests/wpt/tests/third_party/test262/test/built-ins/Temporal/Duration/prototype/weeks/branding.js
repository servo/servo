// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.duration.prototype.weeks
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const weeks = Object.getOwnPropertyDescriptor(Temporal.Duration.prototype, "weeks").get;

assert.sameValue(typeof weeks, "function");

assert.throws(TypeError, () => weeks.call(undefined), "undefined");
assert.throws(TypeError, () => weeks.call(null), "null");
assert.throws(TypeError, () => weeks.call(true), "true");
assert.throws(TypeError, () => weeks.call(""), "empty string");
assert.throws(TypeError, () => weeks.call(Symbol()), "symbol");
assert.throws(TypeError, () => weeks.call(1), "1");
assert.throws(TypeError, () => weeks.call({}), "plain object");
assert.throws(TypeError, () => weeks.call(Temporal.Duration), "Temporal.Duration");
assert.throws(TypeError, () => weeks.call(Temporal.Duration.prototype), "Temporal.Duration.prototype");
