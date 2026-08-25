// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.duration.prototype.round
description: >
  Test that a single day is added when rounding relative to a ZonedDateTime
  with a non-24-hour day length.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Based on a test case by Adam Shaw

var d = new Temporal.Duration(0, 0, 0, 0, 13, 0, 0, 0, 0, 0);
var zdt = Temporal.ZonedDateTime.from('2024-03-10T00:00:00[America/New_York]'); // a 23-hour day

// Rounding 13 hours up to next 12-hour increment gives 24,
// except since `zdt` is 23 hours, an extra day needs to be added
TemporalHelpers.assertDuration(d.round({
  relativeTo: zdt,
  largestUnit: 'years',
  smallestUnit: 'hours',
  roundingIncrement: 12,
  roundingMode: 'ceil'
}), 0, 0, 0, 1, 12, 0, 0, 0, 0, 0);

// If smallestUnit is 'days' instead of 'hours', rounds up to just 1 day
TemporalHelpers.assertDuration(d.round({
  relativeTo: zdt,
  largestUnit: 'years',
  smallestUnit: 'days',
  roundingIncrement: 1,
  roundingMode: 'ceil'
}), 0, 0, 0, 1, 0, 0, 0, 0, 0, 0);

zdt = Temporal.ZonedDateTime.from('2024-11-03T00:00:00[America/New_York]'); // a 25-hour day
d = new Temporal.Duration(0, 0, 0, 0, 25, 0, 0, 0, 0, 0);

// With a 25-hour day and 25-hour duration, rounding up gives exactly 1 day
TemporalHelpers.assertDuration(d.round({
  relativeTo: zdt,
  largestUnit: 'years',
  smallestUnit: 'hours',
  roundingIncrement: 12,
  roundingMode: 'ceil'
}), 0, 0, 0, 1, 0, 0, 0, 0, 0, 0);

d = new Temporal.Duration(0, 0, 0, 0, 24, 0, 0, 0, 0, 0);

// With a 24-hour duration, rounding up gives 24 hours
// (not 1 day, as that would be 25 hours)
TemporalHelpers.assertDuration(d.round({
  relativeTo: zdt,
  largestUnit: 'years',
  smallestUnit: 'hours',
  roundingIncrement: 12,
  roundingMode: 'ceil'
}), 0, 0, 0, 0, 24, 0, 0, 0, 0, 0);

d = new Temporal.Duration(0, 0, 0, 1, 0, 0, 0, 0, 0, 0);

// With a 25-hour day and 1-day duration, rounding up in hours
// rounds from 25 to 36
TemporalHelpers.assertDuration(d.round({
  relativeTo: zdt,
  largestUnit: 'hours',
  smallestUnit: 'hours',
  roundingIncrement: 12,
  roundingMode: 'ceil'
}), 0, 0, 0, 0, 36, 0, 0, 0, 0, 0);

