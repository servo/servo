// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.hoursinday
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const hoursInDay = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "hoursInDay").get;

assert.sameValue(typeof hoursInDay, "function");

assert.throws(TypeError, () => hoursInDay.call(undefined), "undefined");
assert.throws(TypeError, () => hoursInDay.call(null), "null");
assert.throws(TypeError, () => hoursInDay.call(true), "true");
assert.throws(TypeError, () => hoursInDay.call(""), "empty string");
assert.throws(TypeError, () => hoursInDay.call(Symbol()), "symbol");
assert.throws(TypeError, () => hoursInDay.call(1), "1");
assert.throws(TypeError, () => hoursInDay.call({}), "plain object");
assert.throws(TypeError, () => hoursInDay.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => hoursInDay.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
