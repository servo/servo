// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplainyearmonth
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const toPlainYearMonth = Temporal.PlainDate.prototype.toPlainYearMonth;

assert.sameValue(typeof toPlainYearMonth, "function");

assert.throws(TypeError, () => toPlainYearMonth.call(undefined), "undefined");
assert.throws(TypeError, () => toPlainYearMonth.call(null), "null");
assert.throws(TypeError, () => toPlainYearMonth.call(true), "true");
assert.throws(TypeError, () => toPlainYearMonth.call(""), "empty string");
assert.throws(TypeError, () => toPlainYearMonth.call(Symbol()), "symbol");
assert.throws(TypeError, () => toPlainYearMonth.call(1), "1");
assert.throws(TypeError, () => toPlainYearMonth.call({}), "plain object");
assert.throws(TypeError, () => toPlainYearMonth.call(Temporal.PlainDate), "Temporal.PlainDate");
assert.throws(TypeError, () => toPlainYearMonth.call(Temporal.PlainDate.prototype), "Temporal.PlainDate.prototype");
