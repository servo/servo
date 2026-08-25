// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: >
  Check that Buddhist calendar is implemented as proleptic
  (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";

const days3 = new Temporal.Duration(0, 0, 0, -3);
const days3n = new Temporal.Duration(0, 0, 0, 3);
const weeks1 = new Temporal.Duration(0, 0, -1);
const weeks1n = new Temporal.Duration(0, 0, 1);

const date21251004 = Temporal.ZonedDateTime.from({ year: 2125, monthCode: "M10", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date21251015 = Temporal.ZonedDateTime.from({ year: 2125, monthCode: "M10", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
TemporalHelpers.assertPlainDateTime(
  date21251004.subtract(days3).toPlainDateTime(),
  2125, 10, "M10", 7, 12, 34, 0, 0, 0, 0, "add 3 days to 2125-10-04",
  "be", 2125);
TemporalHelpers.assertPlainDateTime(
  date21251015.subtract(days3n).toPlainDateTime(),
  2125, 10, "M10", 12, 12, 34, 0, 0, 0, 0, "subtract 3 days from 2125-10-15",
  "be", 2125);
TemporalHelpers.assertPlainDateTime(
  date21251004.subtract(weeks1).toPlainDateTime(),
  2125, 10, "M10", 11, 12, 34, 0, 0, 0, 0, "add 1 week to 2125-10-04",
  "be", 2125);
TemporalHelpers.assertPlainDateTime(
  date21251015.subtract(weeks1n).toPlainDateTime(),
  2125, 10, "M10", 8, 12, 34, 0, 0, 0, 0, "subtract 1 week from 2125-10-15",
  "be", 2125);

// Test that skipped months in ISO year 1941 are disregarded because the calendar is proleptic

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const months2 = new Temporal.Duration(0, -2);
const months2n = new Temporal.Duration(0, 2);

// 2483 BE = Gregorian year 1940
const date24830301 = Temporal.ZonedDateTime.from({ year: 2483, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date24831201 = Temporal.ZonedDateTime.from({ year: 2483, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date24840301 = Temporal.ZonedDateTime.from({ year: 2484, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date24840416 = Temporal.ZonedDateTime.from({ year: 2484, monthCode: "M04", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });

TemporalHelpers.assertPlainDateTime(
  date24831201.subtract(months2).toPlainDateTime(),
  2484, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "add 2 months to 2484-02-01",
  "be", 2484);
TemporalHelpers.assertPlainDateTime(
  date24840416.subtract(months2n).toPlainDateTime(),
  2484, 2, "M02", 16, 12, 34, 0, 0, 0, 0, "subtract 2 months from 2484-04-16",
  "be", 2484);
TemporalHelpers.assertPlainDateTime(
  date24830301.subtract(years1).toPlainDateTime(),
  2484, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "add 1 years to 2483-03-01",
  "be", 2484);
TemporalHelpers.assertPlainDateTime(
  date24840301.subtract(years1n).toPlainDateTime(),
  2483, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "subtract 1 year from to 2484-03-01",
  "be", 2483);
