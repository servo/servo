// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const toZonedDateTime = Temporal.PlainDate.prototype.toZonedDateTime;

assert.sameValue(typeof toZonedDateTime, "function");

const args = ["UTC"];

assert.throws(TypeError, () => toZonedDateTime.apply(undefined, args), "undefined");
assert.throws(TypeError, () => toZonedDateTime.apply(null, args), "null");
assert.throws(TypeError, () => toZonedDateTime.apply(true, args), "true");
assert.throws(TypeError, () => toZonedDateTime.apply("", args), "empty string");
assert.throws(TypeError, () => toZonedDateTime.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => toZonedDateTime.apply(1, args), "1");
assert.throws(TypeError, () => toZonedDateTime.apply({}, args), "plain object");
assert.throws(TypeError, () => toZonedDateTime.apply(Temporal.PlainDate, args), "Temporal.PlainDate");
assert.throws(TypeError, () => toZonedDateTime.apply(Temporal.PlainDate.prototype, args), "Temporal.PlainDate.prototype");
