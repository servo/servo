// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const until = Temporal.PlainYearMonth.prototype.until;

assert.sameValue(typeof until, "function");

const args = [new Temporal.PlainYearMonth(2022, 6)];

assert.throws(TypeError, () => until.apply(undefined, args), "undefined");
assert.throws(TypeError, () => until.apply(null, args), "null");
assert.throws(TypeError, () => until.apply(true, args), "true");
assert.throws(TypeError, () => until.apply("", args), "empty string");
assert.throws(TypeError, () => until.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => until.apply(1, args), "1");
assert.throws(TypeError, () => until.apply({}, args), "plain object");
assert.throws(TypeError, () => until.apply(Temporal.PlainYearMonth, args), "Temporal.PlainYearMonth");
assert.throws(TypeError, () => until.apply(Temporal.PlainYearMonth.prototype, args), "Temporal.PlainYearMonth.prototype");
