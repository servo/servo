// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: >
  Check various basic calculations not involving leap years or constraining
  (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

const date256407 = Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M07", calendar });

TemporalHelpers.assertPlainYearMonth(
  date256407.add(years1),
  2565, 7, "M07", "add 1y",
  "be", 2565, null);
TemporalHelpers.assertPlainYearMonth(
  date256407.add(years4),
  2568, 7, "M07", "add 4y",
  "be", 2568, null);

TemporalHelpers.assertPlainYearMonth(
  date256407.add(years1n),
  2563, 7, "M07", "subtract 1y",
  "be", 2563, null);
TemporalHelpers.assertPlainYearMonth(
  date256407.add(years4n),
  2560, 7, "M07", "subtract 4y",
  "be", 2560, null);

// Months

const months5 = new Temporal.Duration(0, 5);
const months5n = new Temporal.Duration(0, -5);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

const date255512 = Temporal.PlainYearMonth.from({ year: 2555, monthCode: "M12", calendar });

TemporalHelpers.assertPlainYearMonth(
  date256407.add(months5),
  2564, 12, "M12", "add 5mo with result in the same year",
  "be", 2564, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M08", calendar }).add(months5),
  2565, 1, "M01", "add 5mo with result in the next year",
  "be", 2565, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2562, monthCode: "M10", calendar }).add(months5),
  2563, 3, "M03", "add 5mo with result in the next year on day 1 of month",
  "be", 2563, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M10", calendar }).add(months5),
  2565, 3, "M03", "add 5mo with result in the next year on day 31 of month",
  "be", 2565, null);

TemporalHelpers.assertPlainYearMonth(
  date256407.add(years1months2),
  2565, 9, "M09", "add 1y 2mo",
  "be", 2565, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M11", calendar }).add(years1months2),
  2566, 1, "M01", "add 1y 2mo with result in the next year",
  "be", 2566, null);

TemporalHelpers.assertPlainYearMonth(
  date256407.add(months5n),
  2564, 2, "M02", "subtract 5mo with result in the same year",
  "be", 2564, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M01", calendar }).add(months5n),
  2563, 8, "M08", "subtract 5mo with result in the previous year",
  "be", 2563, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2562, monthCode: "M02", calendar }).add(months5n),
  2561, 9, "M09", "subtract 5mo with result in the previous year on day 1 of month",
  "be", 2561, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M03", calendar }).add(months5n),
  2563, 10, "M10", "subtract 5mo with result in the previous year on day 31 of month",
  "be", 2563, null);

TemporalHelpers.assertPlainYearMonth(
  date256407.add(years1months2n),
  2563, 5, "M05", "subtract 1y 2mo",
  "be", 2563, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M02", calendar }).add(years1months2n),
  2562, 12, "M12", "subtract 1y 2mo with result in the previous year",
  "be", 2562, null);

TemporalHelpers.assertPlainYearMonth(
  date255512.add(months6),
  2556, 6, "M06", "add 6mo",
  "be", 2556, null);
const calculatedStart = date255512.add(months6).add(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  2555, 12, "M12", "subtract 6mo",
  "be", 2555, null);
