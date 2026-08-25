// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: Properties on an object passed to add() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  // ToTemporalDurationRecord
  "get fields.days",
  "get fields.days.valueOf",
  "call fields.days.valueOf",
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
  "get fields.months.valueOf",
  "call fields.months.valueOf",
  "get fields.nanoseconds",
  "get fields.nanoseconds.valueOf",
  "call fields.nanoseconds.valueOf",
  "get fields.seconds",
  "get fields.seconds.valueOf",
  "call fields.seconds.valueOf",
  "get fields.weeks",
  "get fields.weeks.valueOf",
  "call fields.weeks.valueOf",
  "get fields.years",
  "get fields.years.valueOf",
  "call fields.years.valueOf",
];
const actual = [];

const fields = TemporalHelpers.propertyBagObserver(actual, {
  years: 0,
  months: 0,
  weeks: 0,
  days: 1,
  hours: 1,
  minutes: 1,
  seconds: 1,
  milliseconds: 1,
  microseconds: 1,
  nanoseconds: 1,
}, "fields");

// basic order of observable operations, without any calendar units:
const instance = new Temporal.Duration(0, 0, 0, 1, 1, 1, 1, 1, 1, 1);
instance.add(fields);
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear
