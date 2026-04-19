// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.monthcode
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const monthCode = Object.getOwnPropertyDescriptor(Temporal.PlainDateTime.prototype, "monthCode").get;

assert.sameValue(typeof monthCode, "function");

assert.throws(TypeError, () => monthCode.call(undefined), "undefined");
assert.throws(TypeError, () => monthCode.call(null), "null");
assert.throws(TypeError, () => monthCode.call(true), "true");
assert.throws(TypeError, () => monthCode.call(""), "empty string");
assert.throws(TypeError, () => monthCode.call(Symbol()), "symbol");
assert.throws(TypeError, () => monthCode.call(1), "1");
assert.throws(TypeError, () => monthCode.call({}), "plain object");
assert.throws(TypeError, () => monthCode.call(Temporal.PlainDateTime), "Temporal.PlainDateTime");
assert.throws(TypeError, () => monthCode.call(Temporal.PlainDateTime.prototype), "Temporal.PlainDateTime.prototype");
