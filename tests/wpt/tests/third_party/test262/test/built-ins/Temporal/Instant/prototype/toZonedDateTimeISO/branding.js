// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tozoneddatetimeiso
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const toZonedDateTimeISO = Temporal.Instant.prototype.toZonedDateTimeISO;

assert.sameValue(typeof toZonedDateTimeISO, "function");

const args = [{ timeZone: "UTC" }];

assert.throws(TypeError, () => toZonedDateTimeISO.apply(undefined, args), "undefined");
assert.throws(TypeError, () => toZonedDateTimeISO.apply(null, args), "null");
assert.throws(TypeError, () => toZonedDateTimeISO.apply(true, args), "true");
assert.throws(TypeError, () => toZonedDateTimeISO.apply("", args), "empty string");
assert.throws(TypeError, () => toZonedDateTimeISO.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => toZonedDateTimeISO.apply(1, args), "1");
assert.throws(TypeError, () => toZonedDateTimeISO.apply({}, args), "plain object");
assert.throws(TypeError, () => toZonedDateTimeISO.apply(Temporal.Instant, args), "Temporal.Instant");
assert.throws(TypeError, () => toZonedDateTimeISO.apply(Temporal.Instant.prototype, args), "Temporal.Instant.prototype");
