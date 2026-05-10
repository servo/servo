// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-temporal.zoneddatetime.prototype.round
description: Test TZDB edge case where start of day is not 00:00 nor 01:00
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// DST spring-forward hour skipped at 1919-03-30T23:30 (23.5 hour day)
// 11.75 hours is 0.5
const dayBefore = Temporal.ZonedDateTime.from({
  year: 1919,
  month: 3,
  day: 30,
  hour: 11,
  minute: 45,
  timeZone: "America/Toronto",
});
TemporalHelpers.assertPlainDateTime(
  dayBefore.round({ smallestUnit: "day" }).toPlainDateTime(),
  1919, 3, "M03", 31, 0, 30, 0, 0, 0, 0,
  "1919-03-30T11:45 rounds up to start of next day with halfExpand"
);
TemporalHelpers.assertPlainDateTime(
  dayBefore.round({ smallestUnit: "day", roundingMode: "halfTrunc" }).toPlainDateTime(),
  1919, 3, "M03", 30, 0, 0, 0, 0, 0, 0,
  "1919-03-30T11:45 rounds down to start of this day with halfTrunc"
);

// Following day was also 23.5 hours
const dayAfter = Temporal.ZonedDateTime.from({
  year: 1919,
  month: 3,
  day: 31,
  hour: 12,
  minute: 15,
  timeZone: "America/Toronto",
});
TemporalHelpers.assertPlainDateTime(
  dayAfter.round({ smallestUnit: "day" }).toPlainDateTime(),
  1919, 4, "M04", 1, 0, 0, 0, 0, 0, 0,
  "1919-03-31T12:15 rounds up to start of next day with halfExpand"
);
TemporalHelpers.assertPlainDateTime(
  dayAfter.round({ smallestUnit: "day", roundingMode: "halfTrunc" }).toPlainDateTime(),
  1919, 3, "M03", 31, 0, 30, 0, 0, 0, 0,
  "1919-03-31T12:15 rounds down to start of this day with halfTrunc"
);
