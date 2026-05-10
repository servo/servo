// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test various rounding increments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const dCalendar = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const dNoCalendar = new Temporal.Duration(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);
const plainRelativeTo = new Temporal.PlainDate(2020, 1, 1);
const zonedRelativeTo = new Temporal.ZonedDateTime(0n, "UTC");

for (const relativeTo of [plainRelativeTo, zonedRelativeTo]) {
  // Rounds to an increment of hours
  TemporalHelpers.assertDuration(dCalendar.round({
    smallestUnit: "hours",
    roundingIncrement: 3,
    relativeTo
  }), 5, 6, 0, 10, 6, 0, 0, 0, 0, 0);

  // Rounds to an increment of minutes
  TemporalHelpers.assertDuration(dCalendar.round({
    smallestUnit: "minutes",
    roundingIncrement: 30,
    relativeTo
  }), 5, 6, 0, 10, 5, 0, 0, 0, 0, 0);

  // Rounds to an increment of seconds
  TemporalHelpers.assertDuration(dCalendar.round({
    smallestUnit: "seconds",
    roundingIncrement: 15,
    relativeTo
  }), 5, 6, 0, 10, 5, 5, 0, 0, 0, 0);

  // Rounds to an increment of milliseconds
  TemporalHelpers.assertDuration(dCalendar.round({
    smallestUnit: "milliseconds",
    roundingIncrement: 10,
    relativeTo
  }), 5, 6, 0, 10, 5, 5, 5, 10, 0, 0);

  // Rounds to an increment of microseconds
  TemporalHelpers.assertDuration(dCalendar.round({
    smallestUnit: "microseconds",
    roundingIncrement: 10,
    relativeTo
  }), 5, 6, 0, 10, 5, 5, 5, 5, 10, 0);

  // Rounds to an increment of nanoseconds
  TemporalHelpers.assertDuration(dCalendar.round({
    smallestUnit: "nanoseconds",
    roundingIncrement: 10,
    relativeTo
  }), 5, 6, 0, 10, 5, 5, 5, 5, 5, 10);
}

for (const relativeTo of [undefined, plainRelativeTo, zonedRelativeTo]) {
  // Rounds to an increment of days (possible because no higher units than days)
  TemporalHelpers.assertDuration(dNoCalendar.round({
    smallestUnit: "days",
    roundingIncrement: 2,
    relativeTo
  }), 0, 0, 0, 6, 0, 0, 0, 0, 0, 0);

  // Rounds to an increment of hours
  TemporalHelpers.assertDuration(dNoCalendar.round({
    smallestUnit: "hours",
    roundingIncrement: 3,
    relativeTo
  }), 0, 0, 0, 5, 6, 0, 0, 0, 0, 0);

  // Rounds to an increment of minutes
  TemporalHelpers.assertDuration(dNoCalendar.round({
    smallestUnit: "minutes",
    roundingIncrement: 30,
    relativeTo
  }), 0, 0, 0, 5, 5, 0, 0, 0, 0, 0);

  // Rounds to an increment of seconds
  TemporalHelpers.assertDuration(dNoCalendar.round({
    smallestUnit: "seconds",
    roundingIncrement: 15,
    relativeTo
  }), 0, 0, 0, 5, 5, 5, 0, 0, 0, 0);

  // Rounds to an increment of milliseconds
  TemporalHelpers.assertDuration(dNoCalendar.round({
    smallestUnit: "milliseconds",
    roundingIncrement: 10,
    relativeTo
  }), 0, 0, 0, 5, 5, 5, 5, 10, 0, 0);

  // Rounds to an increment of microseconds
  TemporalHelpers.assertDuration(dNoCalendar.round({
    smallestUnit: "microseconds",
    roundingIncrement: 10,
    relativeTo
  }), 0, 0, 0, 5, 5, 5, 5, 5, 10, 0);

  // Rounds to an increment of nanoseconds
  TemporalHelpers.assertDuration(dNoCalendar.round({
    smallestUnit: "nanoseconds",
    roundingIncrement: 10,
    relativeTo
  }), 0, 0, 0, 5, 5, 5, 5, 5, 5, 10);
}
