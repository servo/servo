// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toinstant
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const toInstant = Temporal.ZonedDateTime.prototype.toInstant;

assert.sameValue(typeof toInstant, "function");

assert.throws(TypeError, () => toInstant.call(undefined), "undefined");
assert.throws(TypeError, () => toInstant.call(null), "null");
assert.throws(TypeError, () => toInstant.call(true), "true");
assert.throws(TypeError, () => toInstant.call(""), "empty string");
assert.throws(TypeError, () => toInstant.call(Symbol()), "symbol");
assert.throws(TypeError, () => toInstant.call(1), "1");
assert.throws(TypeError, () => toInstant.call({}), "plain object");
assert.throws(TypeError, () => toInstant.call(Temporal.ZonedDateTime), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => toInstant.call(Temporal.ZonedDateTime.prototype), "Temporal.ZonedDateTime.prototype");
