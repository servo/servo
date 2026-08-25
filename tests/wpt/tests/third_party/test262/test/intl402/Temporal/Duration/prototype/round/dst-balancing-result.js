// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
    Balancing the resulting duration takes the time zone's UTC offset shifts
    into account
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const timeZone = "America/Vancouver";
const skippedHourDay = Temporal.PlainDateTime.from("2000-04-02").toZonedDateTime(timeZone);
const repeatedHourDay = Temporal.PlainDateTime.from("2000-10-29").toZonedDateTime(timeZone);
const inRepeatedHour = new Temporal.ZonedDateTime(972806400_000_000_000n, timeZone);
const oneDay = new Temporal.Duration(0, 0, 0, 1);
const hours12 = new Temporal.Duration(0, 0, 0, 0, 12);
const hours25 = new Temporal.Duration(0, 0, 0, 0, 25);

// Start inside repeated hour, end after.
TemporalHelpers.assertDuration(
  hours25.round({
    largestUnit: "days",
    relativeTo: inRepeatedHour
  }), 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "25 hours in days");

TemporalHelpers.assertDuration(
  oneDay.round({
    largestUnit: "hours",
    relativeTo: inRepeatedHour
  }), 0, 0, 0, 0, 25, 0, 0, 0, 0, 0,
  "25 hours in hours");

// Start after repeated hour, end inside (negative).
const afterRepeatedHour = Temporal.PlainDateTime.from("2000-10-30T01:00").toZonedDateTime(timeZone);
TemporalHelpers.assertDuration(
  hours25.negated().round({
    largestUnit: "days",
    relativeTo: afterRepeatedHour
  }), 0, 0, 0, -1, 0, 0, 0, 0, 0, 0,
  "Negative 25 hours in days");

TemporalHelpers.assertDuration(
  oneDay.negated().round({
    largestUnit: "hours",
    relativeTo: afterRepeatedHour
  }), 0, 0, 0, 0, -25, 0, 0, 0, 0, 0,
  "Negative 25 hours in hours");

// Start inside repeated hour, end in skipped hour.
TemporalHelpers.assertDuration(
  Temporal.Duration.from({
    days: 126,
    hours: 1
  }).round({
    largestUnit: "days",
    relativeTo: inRepeatedHour
  }), 0, 0, 0, 126, 1, 0, 0, 0, 0, 0,
  "Repeated hour to skipped hour, in days");

TemporalHelpers.assertDuration(
  Temporal.Duration.from({
    days: 126,
    hours: 1
  }).round({
    largestUnit: "hours",
    relativeTo: inRepeatedHour
  }), 0, 0, 0, 0, 3026, 0, 0, 0, 0, 0,
  "Repeated hour to skipped hour, in hours");

// Start in normal hour, end in skipped hour.
const inNormalHour = Temporal.PlainDateTime.from("2000-04-01T02:30").toZonedDateTime(timeZone);
TemporalHelpers.assertDuration(
  hours25.round({
    largestUnit: "days",
    relativeTo: inNormalHour
  }), 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
  "Normal hour to skipped hour, in days");

TemporalHelpers.assertDuration(
  oneDay.round({
    largestUnit: "hours",
    relativeTo: inNormalHour
  }), 0, 0, 0, 0, 24, 0, 0, 0, 0, 0,
  "Normal hour to skipped hour, in hours");

// Start before skipped hour, end >1 day after.
TemporalHelpers.assertDuration(
  hours25.round({
    largestUnit: "days",
    relativeTo: skippedHourDay
  }), 0, 0, 0, 1, 2, 0, 0, 0, 0, 0,
  "Before to a day after skipped hour, in days.");

TemporalHelpers.assertDuration(
  oneDay.round({
    largestUnit: "hours",
    relativeTo: skippedHourDay
  }), 0, 0, 0, 0, 23, 0, 0, 0, 0, 0,
  "Before to a day after skipped hour, in hours.");

// Start after skipped hour, end >1 day before (negative).
const afterSkippedHour = Temporal.PlainDateTime.from("2000-04-03T00:00").toZonedDateTime(timeZone);
TemporalHelpers.assertDuration(
  hours25.negated().round({
    largestUnit: "days",
    relativeTo: afterSkippedHour
  }), 0, 0, 0, -1, -2, 0, 0, 0, 0, 0,
  "After to a day before skipped hour, in days.");

TemporalHelpers.assertDuration(
  oneDay.negated().round({
    largestUnit: "hours",
    relativeTo: afterSkippedHour
  }), 0, 0, 0, 0, -23, 0, 0, 0, 0, 0,
  "After to a day before skipped hour, in hours.");

// Start before skipped hour, end <1 day after.
TemporalHelpers.assertDuration(
  hours12.round({
    largestUnit: "days",
    relativeTo: skippedHourDay
  }), 0, 0, 0, 0, 12, 0, 0, 0, 0, 0,
  "Before to less than a day after skipped hour, in days.");

// Start after skipped hour, end <1 day before (negative).
const beforeSkippedHour = Temporal.PlainDateTime.from("2000-04-02T12:00").toZonedDateTime(timeZone);
TemporalHelpers.assertDuration(
  hours12.negated().round({
    largestUnit: "days",
    relativeTo: beforeSkippedHour
  }), 0, 0, 0, 0, -12, 0, 0, 0, 0, 0,
  "After to less than a day before skipped hour, in days.");

// Start before repeated hour, end >1 day after
TemporalHelpers.assertDuration(
  hours25.round({
    largestUnit: "days",
    relativeTo: repeatedHourDay
  }), 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "Before to a day after repeated hour, in days.");

TemporalHelpers.assertDuration(
  oneDay.round({
    largestUnit: "hours",
    relativeTo: repeatedHourDay
  }), 0, 0, 0, 0, 25, 0, 0, 0, 0, 0,
  "Before to a day after repeated hour, in hours.");

// Start after repeated hour, end >1 day before (negative).
const afterRepeatedHour2 = Temporal.PlainDateTime.from("2000-10-30T00:00").toZonedDateTime(timeZone);
TemporalHelpers.assertDuration(
  hours25.negated().round({
    largestUnit: "days",
    relativeTo: afterRepeatedHour2
  }), 0, 0, 0, -1, 0, 0, 0, 0, 0, 0,
  "After to a day before repeated hour, in days.");

TemporalHelpers.assertDuration(
  oneDay.negated().round({
    largestUnit: "hours",
    relativeTo: afterRepeatedHour2
  }), 0, 0, 0, 0, -25, 0, 0, 0, 0, 0,
  "After to a day before repeated hour, in hours.");

// Start before repeated hour, end <1 day after.
TemporalHelpers.assertDuration(
  hours12.round({
    largestUnit: "days",
    relativeTo: repeatedHourDay
  }), 0, 0, 0, 0, 12, 0, 0, 0, 0, 0,
  "Before to less than a day after repeated hour, in days.");

// Start after repeated hour, end <1 day before (negative).
const beforeRepeatedHour = Temporal.PlainDateTime.from("2000-10-29T12:00").toZonedDateTime(timeZone);
TemporalHelpers.assertDuration(
  hours12.negated().round({
    largestUnit: "days",
    relativeTo: beforeRepeatedHour
  }), 0, 0, 0, 0, -12, 0, 0, 0, 0, 0,
  "Before to less than a day after repeated hour, in hours.");

// Samoa skipped 24 hours.
const beforeSkippedDay = Temporal.PlainDateTime.from("2011-12-29T12:00").toZonedDateTime("Pacific/Apia");
TemporalHelpers.assertDuration(
  hours25.round({
    largestUnit: "days",
    relativeTo: beforeSkippedDay
  }), 0, 0, 0, 2, 1, 0, 0, 0, 0, 0,
  "25 hours in days relative to Samoa skipped day.");

TemporalHelpers.assertDuration(
  Temporal.Duration.from({ hours: 48 }).round({
    largestUnit: "days",
    relativeTo: beforeSkippedDay
  }), 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
  "48 hours in days relative to Samoa skipped day.");

// Casts relativeTo to ZonedDateTime if possible.
TemporalHelpers.assertDuration(
  hours25.round({
    largestUnit: "days",
    relativeTo: {
      year: 2000,
      month: 10,
      day: 29,
      timeZone
    }
  }), 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "25 hours in days relativeTo property bag");

// Based on a test case by Adam Shaw
let duration = new Temporal.Duration(1, 0, 0, 0, 24);
let relativeTo = new Temporal.ZonedDateTime(
  941184000_000_000_000n /* = 1999-10-29T08Z */,
  "America/Vancouver"); /* = 1999-10-29T00-08 in local time */

let result = duration.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(result, 1, 0, 0, 0, 24, 0, 0, 0, 0, 0,
  "24 hours does not balance to 1 day in 25-hour day");

duration = new Temporal.Duration(0, 0, 0, 0, /* hours = */ 24, 0, 0, 0, 0, /* ns = */ 5);
relativeTo = new Temporal.ZonedDateTime(
  972802800_000_000_000n /* = 2000-10-29T07Z */,
  "America/Vancouver"); /* = 2000-10-29T00-07 in local time */

result = duration.round({
  largestUnit: "days",
  smallestUnit: "minutes",
  roundingMode: "expand",
  roundingIncrement: 30,
  relativeTo
});
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 24, 30, 0, 0, 0, 0,
  "24 hours does not balance after rounding to 1 day in 25-hour day");
