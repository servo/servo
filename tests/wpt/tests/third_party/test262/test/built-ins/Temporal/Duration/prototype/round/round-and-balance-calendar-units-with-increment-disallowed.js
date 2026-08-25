// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  Disallow rounding to an increment of calendar units >1 if also balancing
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const relativeTo = new Temporal.PlainDate(2024, 1, 1);

const months = Temporal.Duration.from({ months: 9 });

TemporalHelpers.assertDuration(months.round({
  relativeTo,
  smallestUnit: "months",
  roundingIncrement: 8,
  roundingMode: "ceil",
}), 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, "OK to round to an increment of months");

TemporalHelpers.assertDuration(months.round({
  relativeTo,
  largestUnit: "years",
  smallestUnit: "months",
}), 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, "OK to balance to years while rounding to 1 month");

assert.throws(RangeError, () => months.round({
  relativeTo,
  largestUnit: "years",
  smallestUnit: "months",
  roundingIncrement: 8,
  roundingMode: "ceil",
}), "Cannot round to an increment of months while also balancing to years");

const weeks = Temporal.Duration.from({ weeks: 7 });

TemporalHelpers.assertDuration(weeks.round({
  relativeTo,
  smallestUnit: "weeks",
  roundingIncrement: 6,
  roundingMode: "ceil",
}), 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, "OK to round to an increment of weeks");

TemporalHelpers.assertDuration(weeks.round({
  relativeTo,
  largestUnit: "months",
  smallestUnit: "weeks",
}), 0, 1, 3, 0, 0, 0, 0, 0, 0, 0, "OK to balance to months while rounding to 1 week");

assert.throws(RangeError, () => weeks.round({
  relativeTo,
  largestUnit: "months",
  smallestUnit: "weeks",
  roundingIncrement: 6,
  roundingMode: "ceil",
}), "Cannot round to an increment of weeks while also balancing to months");

const days = Temporal.Duration.from({ days: 31 });

TemporalHelpers.assertDuration(days.round({
  relativeTo,
  smallestUnit: "days",
  roundingIncrement: 30,
  roundingMode: "ceil",
}), 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, "OK to round to an increment of days");

TemporalHelpers.assertDuration(days.round({
  relativeTo,
  largestUnit: "weeks",
  smallestUnit: "days",
}), 0, 0, 4, 3, 0, 0, 0, 0, 0, 0, "OK to balance to weeks while rounding to 1 day");

assert.throws(RangeError, () => days.round({
  relativeTo,
  largestUnit: "weeks",
  smallestUnit: "days",
  roundingIncrement: 30,
  roundingMode: "ceil",
}), "Cannot round to an increment of days while also balancing to weeks");
