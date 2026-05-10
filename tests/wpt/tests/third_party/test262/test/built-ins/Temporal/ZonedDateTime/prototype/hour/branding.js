// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.hour
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const hour = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "hour").get;

assert.sameValue(typeof hour, "function");

assert.throws(TypeError, () => hour.call(undefined), "undefined");
assert.throws(TypeError, () => hour.call(null), "null");
assert.throws(TypeError, () => hour.call(true), "true");
assert.throws(TypeError, () => hour.call(""), "empty string");
assert.throws(TypeError, () => hour.call(Symbol()), "symbol");
assert.throws(TypeError, () => hour.call(1), "1");
assert.throws(TypeError, () => hour.call({}), "plain object");
assert.throws(TypeError, () => hour.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => hour.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
