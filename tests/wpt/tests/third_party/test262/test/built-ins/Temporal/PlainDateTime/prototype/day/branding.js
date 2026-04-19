// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.day
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const day = Object.getOwnPropertyDescriptor(Temporal.PlainDateTime.prototype, "day").get;

assert.sameValue(typeof day, "function");

assert.throws(TypeError, () => day.call(undefined), "undefined");
assert.throws(TypeError, () => day.call(null), "null");
assert.throws(TypeError, () => day.call(true), "true");
assert.throws(TypeError, () => day.call(""), "empty string");
assert.throws(TypeError, () => day.call(Symbol()), "symbol");
assert.throws(TypeError, () => day.call(1), "1");
assert.throws(TypeError, () => day.call({}), "plain object");
assert.throws(TypeError, () => day.call(Temporal.PlainDateTime), "Temporal.PlainDateTime");
assert.throws(TypeError, () => day.call(Temporal.PlainDateTime.prototype), "Temporal.PlainDateTime.prototype");
