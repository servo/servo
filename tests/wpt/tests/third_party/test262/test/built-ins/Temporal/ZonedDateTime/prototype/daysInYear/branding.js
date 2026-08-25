// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.daysinyear
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const daysInYear = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "daysInYear").get;

assert.sameValue(typeof daysInYear, "function");

assert.throws(TypeError, () => daysInYear.call(undefined), "undefined");
assert.throws(TypeError, () => daysInYear.call(null), "null");
assert.throws(TypeError, () => daysInYear.call(true), "true");
assert.throws(TypeError, () => daysInYear.call(""), "empty string");
assert.throws(TypeError, () => daysInYear.call(Symbol()), "symbol");
assert.throws(TypeError, () => daysInYear.call(1), "1");
assert.throws(TypeError, () => daysInYear.call({}), "plain object");
assert.throws(TypeError, () => daysInYear.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => daysInYear.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
