// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const getTimeZoneTransition = Temporal.ZonedDateTime.prototype.getTimeZoneTransition;

assert.sameValue(typeof getTimeZoneTransition, "function");

const args = ["next"];

assert.throws(TypeError, () => getTimeZoneTransition.apply(undefined, args), "undefined");
assert.throws(TypeError, () => getTimeZoneTransition.apply(null, args), "null");
assert.throws(TypeError, () => getTimeZoneTransition.apply(true, args), "true");
assert.throws(TypeError, () => getTimeZoneTransition.apply("", args), "empty string");
assert.throws(TypeError, () => getTimeZoneTransition.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => getTimeZoneTransition.apply(1, args), "1");
assert.throws(TypeError, () => getTimeZoneTransition.apply({}, args), "plain object");
assert.throws(TypeError, () => getTimeZoneTransition.apply(Temporal.ZonedDateTime, args), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => getTimeZoneTransition.apply(Temporal.ZonedDateTime.prototype, args), "Temporal.ZonedDateTime.prototype");
