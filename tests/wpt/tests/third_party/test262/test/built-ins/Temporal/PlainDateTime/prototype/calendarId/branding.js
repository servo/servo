// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.calendarid
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const calendarId = Object.getOwnPropertyDescriptor(Temporal.PlainDateTime.prototype, "calendarId").get;

assert.sameValue(typeof calendarId, "function");

assert.throws(TypeError, () => calendarId.call(undefined), "undefined");
assert.throws(TypeError, () => calendarId.call(null), "null");
assert.throws(TypeError, () => calendarId.call(true), "true");
assert.throws(TypeError, () => calendarId.call(""), "empty string");
assert.throws(TypeError, () => calendarId.call(Symbol()), "symbol");
assert.throws(TypeError, () => calendarId.call(1), "1");
assert.throws(TypeError, () => calendarId.call({}), "plain object");
assert.throws(TypeError, () => calendarId.call(Temporal.PlainDateTime), "Temporal.PlainDateTime");
assert.throws(TypeError, () => calendarId.call(Temporal.PlainDateTime.prototype), "Temporal.PlainDateTime.prototype");
