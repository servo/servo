// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.timezoneid
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const timeZoneId = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "timeZoneId").get;

assert.sameValue(typeof timeZoneId, "function");

assert.throws(TypeError, () => timeZoneId.call(undefined), "undefined");
assert.throws(TypeError, () => timeZoneId.call(null), "null");
assert.throws(TypeError, () => timeZoneId.call(true), "true");
assert.throws(TypeError, () => timeZoneId.call(""), "empty string");
assert.throws(TypeError, () => timeZoneId.call(Symbol()), "symbol");
assert.throws(TypeError, () => timeZoneId.call(1), "1");
assert.throws(TypeError, () => timeZoneId.call({}), "plain object");
assert.throws(TypeError, () => timeZoneId.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => timeZoneId.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
