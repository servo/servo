// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.subtract
description: Properties on an object passed to subtract() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Instant(10n);
const expected = [
  "get fields.days",
  "get fields.hours",
  "get fields.hours.valueOf",
  "call fields.hours.valueOf",
  "get fields.microseconds",
  "get fields.microseconds.valueOf",
  "call fields.microseconds.valueOf",
  "get fields.milliseconds",
  "get fields.milliseconds.valueOf",
  "call fields.milliseconds.valueOf",
  "get fields.minutes",
  "get fields.minutes.valueOf",
  "call fields.minutes.valueOf",
  "get fields.months",
  "get fields.nanoseconds",
  "get fields.nanoseconds.valueOf",
  "call fields.nanoseconds.valueOf",
  "get fields.seconds",
  "get fields.seconds.valueOf",
  "call fields.seconds.valueOf",
  "get fields.weeks",
  "get fields.years",
];
const actual = [];
const fields = TemporalHelpers.propertyBagObserver(actual, {
  hours: 1,
  minutes: 1,
  seconds: 1,
  milliseconds: 1,
  microseconds: 1,
  nanoseconds: 1,
}, "fields");
const result = instance.subtract(fields);
assert.sameValue(result.epochNanoseconds, -3661001000991n, "epochNanoseconds result");
assert.compareArray(actual, expected, "order of operations");
