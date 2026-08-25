// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: >
  Check that Buddhist calendar is implemented as proleptic
  (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";

const days3 = new Temporal.Duration(0, 0, 0, 3);
const days3n = new Temporal.Duration(0, 0, 0, -3);
const weeks1 = new Temporal.Duration(0, 0, 1);
const weeks1n = new Temporal.Duration(0, 0, -1);

const date21251004 = Temporal.PlainDate.from({ year: 2125, monthCode: "M10", day: 4, calendar });
const date21251015 = Temporal.PlainDate.from({ year: 2125, monthCode: "M10", day: 15, calendar });
TemporalHelpers.assertPlainDate(
  date21251004.add(days3),
  2125, 10, "M10", 7, "add 3 days to 2125-10-04",
  "be", 2125);
TemporalHelpers.assertPlainDate(
  date21251015.add(days3n),
  2125, 10, "M10", 12, "subtract 3 days from 2125-10-15",
  "be", 2125);
TemporalHelpers.assertPlainDate(
  date21251004.add(weeks1),
  2125, 10, "M10", 11, "add 1 week to 2125-10-04",
  "be", 2125);
TemporalHelpers.assertPlainDate(
  date21251015.add(weeks1n),
  2125, 10, "M10", 8, "subtract 1 week from 2125-10-15",
  "be", 2125);

// Test that skipped months in ISO year 1941 are disregarded because the calendar is proleptic

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const months2 = new Temporal.Duration(0, 2);
const months2n = new Temporal.Duration(0, -2);

// 2483 BE = Gregorian year 1940
const date24830301 = Temporal.PlainDate.from({ year: 2483, monthCode: "M03", day: 1, calendar });
const date24831201 = Temporal.PlainDate.from({ year: 2483, monthCode: "M12", day: 1, calendar });
const date24840301 = Temporal.PlainDate.from({ year: 2484, monthCode: "M03", day: 1, calendar });
const date24840416 = Temporal.PlainDate.from({ year: 2484, monthCode: "M04", day: 16, calendar });

TemporalHelpers.assertPlainDate(
  date24831201.add(months2),
  2484, 2, "M02", 1, "add 2 months to 2484-02-01",
  "be", 2484);
TemporalHelpers.assertPlainDate(
  date24840416.add(months2n),
  2484, 2, "M02", 16, "subtract 2 months from 2484-04-16",
  "be", 2484);
TemporalHelpers.assertPlainDate(
  date24830301.add(years1),
  2484, 3, "M03", 1, "add 1 years to 2483-03-01",
  "be", 2484);
TemporalHelpers.assertPlainDate(
  date24840301.add(years1n),
  2483, 3, "M03", 1, "subtract 1 year from to 2484-03-01",
  "be", 2483);
