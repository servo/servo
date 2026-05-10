// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.withcalendar
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const withCalendar = Temporal.PlainDate.prototype.withCalendar;

assert.sameValue(typeof withCalendar, "function");

const args = ["iso8601"];

assert.throws(TypeError, () => withCalendar.apply(undefined, args), "undefined");
assert.throws(TypeError, () => withCalendar.apply(null, args), "null");
assert.throws(TypeError, () => withCalendar.apply(true, args), "true");
assert.throws(TypeError, () => withCalendar.apply("", args), "empty string");
assert.throws(TypeError, () => withCalendar.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => withCalendar.apply(1, args), "1");
assert.throws(TypeError, () => withCalendar.apply({}, args), "plain object");
assert.throws(TypeError, () => withCalendar.apply(Temporal.PlainDate, args), "Temporal.PlainDate");
assert.throws(TypeError, () => withCalendar.apply(Temporal.PlainDate.prototype, args), "Temporal.PlainDate.prototype");
