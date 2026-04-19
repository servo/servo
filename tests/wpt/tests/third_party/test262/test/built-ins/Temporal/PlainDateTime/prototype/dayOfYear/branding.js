// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.dayofyear
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const dayOfYear = Object.getOwnPropertyDescriptor(Temporal.PlainDateTime.prototype, "dayOfYear").get;

assert.sameValue(typeof dayOfYear, "function");

assert.throws(TypeError, () => dayOfYear.call(undefined), "undefined");
assert.throws(TypeError, () => dayOfYear.call(null), "null");
assert.throws(TypeError, () => dayOfYear.call(true), "true");
assert.throws(TypeError, () => dayOfYear.call(""), "empty string");
assert.throws(TypeError, () => dayOfYear.call(Symbol()), "symbol");
assert.throws(TypeError, () => dayOfYear.call(1), "1");
assert.throws(TypeError, () => dayOfYear.call({}), "plain object");
assert.throws(TypeError, () => dayOfYear.call(Temporal.PlainDateTime), "Temporal.PlainDateTime");
assert.throws(TypeError, () => dayOfYear.call(Temporal.PlainDateTime.prototype), "Temporal.PlainDateTime.prototype");
