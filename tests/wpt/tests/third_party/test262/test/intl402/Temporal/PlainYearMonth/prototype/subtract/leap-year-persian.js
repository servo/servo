// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Check various basic calculations involving leap years (Persian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "persian";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

const date136212 = Temporal.PlainYearMonth.from({ year: 1362, monthCode: "M12", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date136212.subtract(years1, options),
  1363, 12, "M12", "add 1y in leap year",
  "ap", 1363, null);

TemporalHelpers.assertPlainYearMonth(
  date136212.subtract(years4, options),
  1366, 12, "M12", "add 4y in leap year",
  "ap", 1366, null);

TemporalHelpers.assertPlainYearMonth(
  date136212.subtract(years1n, options),
  1361, 12, "M12", "subtract 1y in leap year",
  "ap", 1361, null);

TemporalHelpers.assertPlainYearMonth(
  date136212.subtract(years4n, options),
  1358, 12, "M12", "subtract 4y in leap year",
  "ap", 1358, null);

// Months

const months1n = new Temporal.Duration(0, 1);
const months6 = new Temporal.Duration(0, -6);
const months11n = new Temporal.Duration(0, 11);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date136206 = Temporal.PlainYearMonth.from({ year: 1362, monthCode: "M06", calendar }, options);
const date136211 = Temporal.PlainYearMonth.from({ year: 1362, monthCode: "M11", calendar }, options);
const date136301 = Temporal.PlainYearMonth.from({ year: 1363, monthCode: "M01", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date136206.subtract(months6, options),
  1362, 12, "M12", "add 6mo to Shahrivar in leap year",
  "ap", 1362, null);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1362, monthCode: "M10", calendar }, options).subtract(years1months2, options),
  1363, 12, "M12", "add 1y 2mo with result in the next year",
  "ap", 1363, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1361, monthCode: "M10", calendar }, options).subtract(years1months2),
  1362, 12, "M12", "add 1y 2mo with result in the next leap year",
  "ap", 1362, null);

TemporalHelpers.assertPlainYearMonth(
  date136301.subtract(months1n, options),
  1362, 12, "M12", "subtract 1mo from Farvardin in leap year",
  "ap", 1362, null);

TemporalHelpers.assertPlainYearMonth(
  date136211.subtract(months11n, options),
  1361, 12, "M12", "subtract 11mo with result in the previous year",
  "ap", 1361, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1363, monthCode: "M11", calendar }, options).subtract(months11n, options),
  1362, 12, "M12", "subtract 11mo with result in the previous leap year",
  "ap", 1362, null);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1364, monthCode: "M02", calendar }, options).subtract(years1months2n, options),
  1362, 12, "M12", "add 1y 2mo with result in the previous year",
  "ap", 1362, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1365, monthCode: "M02", calendar }, options).subtract(years1months2n, options),
  1363, 12, "M12", "add 1y 2mo with result in the previous leap year",
  "ap", 1363, null);
