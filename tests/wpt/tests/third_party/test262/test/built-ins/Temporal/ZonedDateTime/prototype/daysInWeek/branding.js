// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.daysinweek
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const daysInWeek = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "daysInWeek").get;

assert.sameValue(typeof daysInWeek, "function");

assert.throws(TypeError, () => daysInWeek.call(undefined), "undefined");
assert.throws(TypeError, () => daysInWeek.call(null), "null");
assert.throws(TypeError, () => daysInWeek.call(true), "true");
assert.throws(TypeError, () => daysInWeek.call(""), "empty string");
assert.throws(TypeError, () => daysInWeek.call(Symbol()), "symbol");
assert.throws(TypeError, () => daysInWeek.call(1), "1");
assert.throws(TypeError, () => daysInWeek.call({}), "plain object");
assert.throws(TypeError, () => daysInWeek.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => daysInWeek.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
