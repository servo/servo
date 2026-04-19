// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: >
  Check various basic calculations not involving leap years or constraining
  (japanese calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "japanese";

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

const date202107 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M07", calendar });

TemporalHelpers.assertPlainYearMonth(
  date202107.add(years1),
  2022, 7, "M07", "add 1y",
  "reiwa", 4);
TemporalHelpers.assertPlainYearMonth(
  date202107.add(years4),
  2025, 7, "M07", "add 4y",
  "reiwa", 7);

TemporalHelpers.assertPlainYearMonth(
  date202107.add(years1n),
  2020, 7, "M07", "subtract 1y",
  "reiwa", 2);
TemporalHelpers.assertPlainYearMonth(
  date202107.add(years4n),
  2017, 7, "M07", "subtract 4y",
  "heisei", 29);

// Months

const months5 = new Temporal.Duration(0, 5);
const months5n = new Temporal.Duration(0, -5);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

const date20001201 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M12", calendar });

TemporalHelpers.assertPlainYearMonth(
  date202107.add(months5),
  2021, 12, "M12", "add 5mo with result in the same year",
  "reiwa", 3);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M08", calendar }).add(months5),
  2022, 1, "M01", "add 5mo with result in the next year",
  "reiwa", 4);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M10", calendar }).add(months5),
  2020, 3, "M03", "add 5mo with result in the next year on day 1 of month",
  "reiwa", 2);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M10", calendar }).add(months5),
  2022, 3, "M03", "add 5mo with result in the next year on day 31 of month",
  "reiwa", 4);

TemporalHelpers.assertPlainYearMonth(
  date202107.add(years1months2),
  2022, 9, "M09", "add 1y 2mo",
  "reiwa", 4);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M11", calendar }).add(years1months2),
  2023, 1, "M01", "add 1y 2mo with result in the next year",
  "reiwa", 5);

TemporalHelpers.assertPlainYearMonth(
  date202107.add(months5n),
  2021, 2, "M02", "subtract 5mo with result in the same year",
  "reiwa", 3);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M01", calendar }).add(months5n),
  2020, 8, "M08", "subtract 5mo with result in the previous year",
  "reiwa", 2);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M02", calendar }).add(months5n),
  2018, 9, "M09", "subtract 5mo with result in the previous year on day 1 of month",
  "heisei", 30);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M03", calendar }).add(months5n),
  2020, 10, "M10", "subtract 5mo with result in the previous year on day 31 of month",
  "reiwa", 2);

TemporalHelpers.assertPlainYearMonth(
  date202107.add(years1months2n),
  2020, 5, "M05", "subtract 1y 2mo",
  "reiwa", 2);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M02", calendar }).add(years1months2n),
  2019, 12, "M12", "subtract 1y 2mo with result in the previous year",
  "reiwa", 1);

TemporalHelpers.assertPlainYearMonth(
  date20001201.add(months6),
  2001, 6, "M06", "add 6 months, with result in next year",
  "heisei", 13);
const calculatedStart = date20001201.add(months6).add(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  2000, 12, "M12", "subtract 6 months, with result in previous year",
  "heisei", 12);
