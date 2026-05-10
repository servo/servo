// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaintime
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const toPlainTime = Temporal.ZonedDateTime.prototype.toPlainTime;

assert.sameValue(typeof toPlainTime, "function");

assert.throws(TypeError, () => toPlainTime.call(undefined), "undefined");
assert.throws(TypeError, () => toPlainTime.call(null), "null");
assert.throws(TypeError, () => toPlainTime.call(true), "true");
assert.throws(TypeError, () => toPlainTime.call(""), "empty string");
assert.throws(TypeError, () => toPlainTime.call(Symbol()), "symbol");
assert.throws(TypeError, () => toPlainTime.call(1), "1");
assert.throws(TypeError, () => toPlainTime.call({}), "plain object");
assert.throws(TypeError, () => toPlainTime.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => toPlainTime.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
