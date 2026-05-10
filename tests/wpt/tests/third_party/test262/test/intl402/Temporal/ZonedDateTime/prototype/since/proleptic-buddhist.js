// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: >
  Check that Buddhist calendar is implemented as proleptic
  (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";

const date21251004 = Temporal.ZonedDateTime.from({ year: 2125, monthCode: "M10", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date21251007 = Temporal.ZonedDateTime.from({ year: 2125, monthCode: "M10", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date21251011 = Temporal.ZonedDateTime.from({ year: 2125, monthCode: "M10", day: 11, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date21251012 = Temporal.ZonedDateTime.from({ year: 2125, monthCode: "M10", day: 12, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date21251015 = Temporal.ZonedDateTime.from({ year: 2125, monthCode: "M10", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
TemporalHelpers.assertDuration(
  date21251004.since(date21251007, { largestUnit: "days" }),
  0, 0, 0, -3, 0, 0, 0, 0, 0, 0,
  "2125-10-04 and 2125-10-07");
TemporalHelpers.assertDuration(
  date21251015.since(date21251012, { largestUnit: "days" }),
  0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
  "2125-10-15 and 2125-10-12");
TemporalHelpers.assertDuration(
  date21251004.since(date21251011, { largestUnit: "weeks" }),
  0, 0, -1, 0, 0, 0, 0, 0, 0, 0,
  "2125-10-04 and 2125-10-11")
TemporalHelpers.assertDuration(
  date21251011.since(date21251004, { largestUnit: "weeks" }),
  0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
  "2125-10-11 and 2125-10-04")

// Test that skipped months in ISO year 1941 are disregarded because the calendar is proleptic

// 2483 BE = Gregorian year 1940
const date24830301 = Temporal.ZonedDateTime.from({ year: 2483, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date24831201 = Temporal.ZonedDateTime.from({ year: 2483, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date24840201 = Temporal.ZonedDateTime.from({ year: 2484, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date24840216 = Temporal.ZonedDateTime.from({ year: 2484, monthCode: "M02", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date24840301 = Temporal.ZonedDateTime.from({ year: 2484, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date24840416 = Temporal.ZonedDateTime.from({ year: 2484, monthCode: "M04", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });

// 2483 is a leap year => 3 months
TemporalHelpers.assertDuration(
  date24831201.since(date24840201, { largestUnit: "months" }),
  0, -3, 0, 0, 0, 0, 0, 0, 0, 0,
  "2483-12-01 and 2484-02-01"
);
TemporalHelpers.assertDuration(
  date24840416.since(date24840216, { largestUnit: "months" }),
  0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
  "2484-04-16 and 2484-02-16"
);
TemporalHelpers.assertDuration(
  date24830301.since(date24840301, { largestUnit: "years" }),
  -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "2483-03-01 and 2484-03-01"
);
TemporalHelpers.assertDuration(
  date24830301.since(date24840301, { largestUnit: "years" }),
  -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "2484-03-01 and 2483-03-01"
);
