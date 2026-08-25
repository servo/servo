// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.equals
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const equals = Temporal.PlainMonthDay.prototype.equals;

assert.sameValue(typeof equals, "function");

const args = [new Temporal.PlainMonthDay(5, 2)];

assert.throws(TypeError, () => equals.apply(undefined, args), "undefined");
assert.throws(TypeError, () => equals.apply(null, args), "null");
assert.throws(TypeError, () => equals.apply(true, args), "true");
assert.throws(TypeError, () => equals.apply("", args), "empty string");
assert.throws(TypeError, () => equals.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => equals.apply(1, args), "1");
assert.throws(TypeError, () => equals.apply({}, args), "plain object");
assert.throws(TypeError, () => equals.apply(Temporal.PlainMonthDay, args), "Temporal.PlainMonthDay");
assert.throws(TypeError, () => equals.apply(Temporal.PlainMonthDay.prototype, args), "Temporal.PlainMonthDay.prototype");
