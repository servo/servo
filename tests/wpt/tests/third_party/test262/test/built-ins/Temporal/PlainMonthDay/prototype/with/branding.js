// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const with_ = Temporal.PlainMonthDay.prototype.with;

assert.sameValue(typeof with_, "function");

const args = [{ year: 2022, month: 12 }];

assert.throws(TypeError, () => with_.apply(undefined, args), "undefined");
assert.throws(TypeError, () => with_.apply(null, args), "null");
assert.throws(TypeError, () => with_.apply(true, args), "true");
assert.throws(TypeError, () => with_.apply("", args), "empty string");
assert.throws(TypeError, () => with_.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => with_.apply(1, args), "1");
assert.throws(TypeError, () => with_.apply({}, args), "plain object");
assert.throws(TypeError, () => with_.apply(Temporal.PlainMonthDay, args), "Temporal.PlainMonthDay");
assert.throws(TypeError, () => with_.apply(Temporal.PlainMonthDay.prototype, args), "Temporal.PlainMonthDay.prototype");
