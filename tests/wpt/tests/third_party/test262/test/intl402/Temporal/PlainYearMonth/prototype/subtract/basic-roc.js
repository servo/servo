// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: >
  Check various basic calculations not involving leap years or constraining
  (roc calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "roc";

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

const date1110716 = Temporal.PlainYearMonth.from({ year: 111, monthCode: "M07", calendar });

TemporalHelpers.assertPlainYearMonth(
  date1110716.subtract(years1),
  112, 7, "M07", "add 1y",
  "roc", 112);
TemporalHelpers.assertPlainYearMonth(
  date1110716.subtract(years4),
  115, 7, "M07", "add 4y",
  "roc", 115);

TemporalHelpers.assertPlainYearMonth(
  date1110716.subtract(years1n),
  110, 7, "M07", "subtract 1y",
  "roc", 110);
TemporalHelpers.assertPlainYearMonth(
  date1110716.subtract(years4n),
  107, 7, "M07", "subtract 4y",
  "roc", 107);

// Months

const months5 = new Temporal.Duration(0, -5);
const months5n = new Temporal.Duration(0, 5);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date901201 = Temporal.PlainYearMonth.from({ year: 90, monthCode: "M12", calendar });

TemporalHelpers.assertPlainYearMonth(
  date1110716.subtract(months5),
  111, 12, "M12", "add 5mo with result in the same year",
  "roc", 111);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 111, monthCode: "M08", calendar }).subtract(months5),
  112, 1, "M01", "add 5mo with result in the next year",
  "roc", 112);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 109, monthCode: "M10", calendar }).subtract(months5),
  110, 3, "M03", "add 5mo with result in the next year on day 1 of month",
  "roc", 110);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 111, monthCode: "M10", calendar }).subtract(months5),
  112, 3, "M03", "add 5mo with result in the next year on day 31 of month",
  "roc", 112);

TemporalHelpers.assertPlainYearMonth(
  date1110716.subtract(years1months2),
  112, 9, "M09", "add 1y 2mo",
  "roc", 112);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 111, monthCode: "M11", calendar }).subtract(years1months2),
  113, 1, "M01", "add 1y 2mo with result in the next year",
  "roc", 113);

TemporalHelpers.assertPlainYearMonth(
  date1110716.subtract(months5n),
  111, 2, "M02", "subtract 5mo with result in the same year",
  "roc", 111);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 111, monthCode: "M01", calendar }).subtract(months5n),
  110, 8, "M08", "subtract 5mo with result in the previous year",
  "roc", 110);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 109, monthCode: "M02", calendar }).subtract(months5n),
  108, 9, "M09", "subtract 5mo with result in the previous year on day 1 of month",
  "roc", 108);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 111, monthCode: "M03", calendar }).subtract(months5n),
  110, 10, "M10", "subtract 5mo with result in the previous year on day 31 of month",
  "roc", 110);

TemporalHelpers.assertPlainYearMonth(
  date1110716.subtract(years1months2n),
  110, 5, "M05", "subtract 1y 2mo",
  "roc", 110);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 111, monthCode: "M02", calendar }).subtract(years1months2n),
  109, 12, "M12", "subtract 1y 2mo with result in the previous year",
  "roc", 109);

TemporalHelpers.assertPlainYearMonth(
  date901201.subtract(months6),
  91, 6, "M06", "add 6 months, with result in next year",
  "roc", 91);
const calculatedStart = date901201.subtract(months6).subtract(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  90, 12, "M12", "subtract 6 months, with result in previous year",
  "roc", 90);
