// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const equals = Temporal.ZonedDateTime.prototype.equals;

assert.sameValue(typeof equals, "function");

const args = [new Temporal.ZonedDateTime(123456n, "UTC")];

assert.throws(TypeError, () => equals.apply(undefined, args), "undefined");
assert.throws(TypeError, () => equals.apply(null, args), "null");
assert.throws(TypeError, () => equals.apply(true, args), "true");
assert.throws(TypeError, () => equals.apply("", args), "empty string");
assert.throws(TypeError, () => equals.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => equals.apply(1, args), "1");
assert.throws(TypeError, () => equals.apply({}, args), "plain object");
assert.throws(TypeError, () => equals.apply(Temporal.ZonedDateTime, args), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => equals.apply(Temporal.ZonedDateTime.prototype, args), "Temporal.ZonedDateTime.prototype");
