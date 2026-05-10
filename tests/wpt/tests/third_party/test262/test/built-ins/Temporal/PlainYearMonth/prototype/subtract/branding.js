// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const subtract = Temporal.PlainYearMonth.prototype.subtract;

assert.sameValue(typeof subtract, "function");

const args = [new Temporal.Duration(5)];

assert.throws(TypeError, () => subtract.apply(undefined, args), "undefined");
assert.throws(TypeError, () => subtract.apply(null, args), "null");
assert.throws(TypeError, () => subtract.apply(true, args), "true");
assert.throws(TypeError, () => subtract.apply("", args), "empty string");
assert.throws(TypeError, () => subtract.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => subtract.apply(1, args), "1");
assert.throws(TypeError, () => subtract.apply({}, args), "plain object");
assert.throws(TypeError, () => subtract.apply(Temporal.PlainYearMonth, args), "Temporal.PlainYearMonth");
assert.throws(TypeError, () => subtract.apply(Temporal.PlainYearMonth.prototype, args), "Temporal.PlainYearMonth.prototype");
