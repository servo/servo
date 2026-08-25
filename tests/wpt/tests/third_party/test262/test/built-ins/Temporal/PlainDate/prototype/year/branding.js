// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.year
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const year = Object.getOwnPropertyDescriptor(Temporal.PlainDate.prototype, "year").get;

assert.sameValue(typeof year, "function");

assert.throws(TypeError, () => year.call(undefined), "undefined");
assert.throws(TypeError, () => year.call(null), "null");
assert.throws(TypeError, () => year.call(true), "true");
assert.throws(TypeError, () => year.call(""), "empty string");
assert.throws(TypeError, () => year.call(Symbol()), "symbol");
assert.throws(TypeError, () => year.call(1), "1");
assert.throws(TypeError, () => year.call({}), "plain object");
assert.throws(TypeError, () => year.call(Temporal.PlainDate), "Temporal.PlainDate");
assert.throws(TypeError, () => year.call(Temporal.PlainDate.prototype), "Temporal.PlainDate.prototype");
