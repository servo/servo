// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: >
  Check various basic calculations not involving leap years or constraining
  (indian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "indian";

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

const date192007 = Temporal.PlainYearMonth.from({ year: 1920, monthCode: "M07", calendar });

TemporalHelpers.assertPlainYearMonth(
  date192007.subtract(years1),
  1921, 7, "M07", "add 1y",
  "shaka", 1921, null);
TemporalHelpers.assertPlainYearMonth(
  date192007.subtract(years4),
  1924, 7, "M07", "add 4y",
  "shaka", 1924, null);

TemporalHelpers.assertPlainYearMonth(
  date192007.subtract(years1n),
  1919, 7, "M07", "subtract 1y",
  "shaka", 1919, null);
TemporalHelpers.assertPlainYearMonth(
  date192007.subtract(years4n),
  1916, 7, "M07", "subtract 4y",
  "shaka", 1916, null);

// Months

const months5 = new Temporal.Duration(0, -5);
const months5n = new Temporal.Duration(0, 5);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);
const months8 = new Temporal.Duration(0, -8);
const months8n = new Temporal.Duration(0, 8);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date192212 = Temporal.PlainYearMonth.from({ year: 1922, monthCode: "M12", calendar });

TemporalHelpers.assertPlainYearMonth(
  date192007.subtract(months5),
  1920, 12, "M12", "add 5mo with result in the same year",
  "shaka", 1920, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1920, monthCode: "M08", calendar }).subtract(months5),
  1921, 1, "M01", "add 5mo with result in the next year",
  "shaka", 1921, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1918, monthCode: "M10", calendar }).subtract(months5),
  1919, 3, "M03", "add 5mo with result in the next year on day 1 of month",
  "shaka", 1919, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1920, monthCode: "M06", calendar }).subtract(months8),
  1921, 2, "M02", "add 8mo with result in the next year on day 31 of month",
  "shaka", 1921, null);

TemporalHelpers.assertPlainYearMonth(
  date192007.subtract(years1months2),
  1921, 9, "M09", "add 1y 2mo",
  "shaka", 1921, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1920, monthCode: "M11", calendar }).subtract(years1months2),
  1922, 1, "M01", "add 1y 2mo with result in the next year",
  "shaka", 1922, null);

TemporalHelpers.assertPlainYearMonth(
  date192007.subtract(months5n),
  1920, 2, "M02", "subtract 5mo with result in the same year",
  "shaka", 1920, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1920, monthCode: "M01", calendar }).subtract(months5n),
  1919, 8, "M08", "subtract 5mo with result in the previous year",
  "shaka", 1919, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1918, monthCode: "M02", calendar }).subtract(months5n),
  1917, 9, "M09", "subtract 5mo with result in the previous year on day 1 of month",
  "shaka", 1917, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1920, monthCode: "M02", calendar }).subtract(months8n),
  1919, 6, "M06", "subtract 8mo with result in the previous year on day 31 of month",
  "shaka", 1919, null);

TemporalHelpers.assertPlainYearMonth(
  date192007.subtract(years1months2n),
  1919, 5, "M05", "subtract 1y 2mo",
  "shaka", 1919, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1920, monthCode: "M02", calendar }).subtract(years1months2n),
  1918, 12, "M12", "subtract 1y 2mo with result in the previous year",
  "shaka", 1918, null);

TemporalHelpers.assertPlainYearMonth(
  date192212.subtract(months6),
  1923, 6, "M06", "add 6 months, with result in next year",
  "shaka", 1923, null);
const calculatedStart = date192212.subtract(months6).subtract(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  1922, 12, "M12", "subtract 6 months, with result in previous year",
  "shaka", 1922, null);
