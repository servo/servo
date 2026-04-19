// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.startofday
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const startOfDay = Temporal.ZonedDateTime.prototype.startOfDay;

assert.sameValue(typeof startOfDay, "function");

assert.throws(TypeError, () => startOfDay.call(undefined), "undefined");
assert.throws(TypeError, () => startOfDay.call(null), "null");
assert.throws(TypeError, () => startOfDay.call(true), "true");
assert.throws(TypeError, () => startOfDay.call(""), "empty string");
assert.throws(TypeError, () => startOfDay.call(Symbol()), "symbol");
assert.throws(TypeError, () => startOfDay.call(1), "1");
assert.throws(TypeError, () => startOfDay.call({}), "plain object");
assert.throws(TypeError, () => startOfDay.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => startOfDay.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
