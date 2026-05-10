// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.month
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const month = Object.getOwnPropertyDescriptor(Temporal.PlainDate.prototype, "month").get;

assert.sameValue(typeof month, "function");

assert.throws(TypeError, () => month.call(undefined), "undefined");
assert.throws(TypeError, () => month.call(null), "null");
assert.throws(TypeError, () => month.call(true), "true");
assert.throws(TypeError, () => month.call(""), "empty string");
assert.throws(TypeError, () => month.call(Symbol()), "symbol");
assert.throws(TypeError, () => month.call(1), "1");
assert.throws(TypeError, () => month.call({}), "plain object");
assert.throws(TypeError, () => month.call(Temporal.PlainDate), "Temporal.PlainDate");
assert.throws(TypeError, () => month.call(Temporal.PlainDate.prototype), "Temporal.PlainDate.prototype");
