// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.dayofweek
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const dayOfWeek = Object.getOwnPropertyDescriptor(Temporal.PlainDateTime.prototype, "dayOfWeek").get;

assert.sameValue(typeof dayOfWeek, "function");

assert.throws(TypeError, () => dayOfWeek.call(undefined), "undefined");
assert.throws(TypeError, () => dayOfWeek.call(null), "null");
assert.throws(TypeError, () => dayOfWeek.call(true), "true");
assert.throws(TypeError, () => dayOfWeek.call(""), "empty string");
assert.throws(TypeError, () => dayOfWeek.call(Symbol()), "symbol");
assert.throws(TypeError, () => dayOfWeek.call(1), "1");
assert.throws(TypeError, () => dayOfWeek.call({}), "plain object");
assert.throws(TypeError, () => dayOfWeek.call(Temporal.PlainDateTime), "Temporal.PlainDateTime");
assert.throws(TypeError, () => dayOfWeek.call(Temporal.PlainDateTime.prototype), "Temporal.PlainDateTime.prototype");
