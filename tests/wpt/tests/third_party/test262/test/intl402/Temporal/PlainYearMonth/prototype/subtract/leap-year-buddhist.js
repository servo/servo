// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Check various basic calculations involving leap years (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

const date256302 = Temporal.PlainYearMonth.from({ year: 2563, monthCode: "M02", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date256302.subtract(years1, options),
  2564, 2, "M02", "add 1y to Feb",
  "be", 2564);
TemporalHelpers.assertPlainYearMonth(
  date256302.subtract(years4, options),
  2567, 2, "M02", "add 4y to Feb",
  "be", 2567);

TemporalHelpers.assertPlainYearMonth(
  date256302.subtract(years1n, options),
  2562, 2, "M02", "subtract 1y from Feb",
  "be", 2562);
TemporalHelpers.assertPlainYearMonth(
  date256302.subtract(years4n, options),
  2559, 2, "M02", "subtract 4y from Feb",
  "be", 2559);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months5 = new Temporal.Duration(0, -5);
const months11n = new Temporal.Duration(0, 11);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date256301 = Temporal.PlainYearMonth.from({ year: 2563, monthCode: "M01", calendar }, options);
const date256303 = Temporal.PlainYearMonth.from({ year: 2563, monthCode: "M03", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date256301.subtract(months1, options),
  2563, 2, "M02", "add 1mo to Jan in leap year",
  "be", 2563);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M09", calendar }, options).subtract(months5),
  2565, 2, "M02", "add 5mo with result in the next year",
  "be", 2565);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2562, monthCode: "M09", calendar }, options).subtract(months5),
  2563, 2, "M02", "add 5mo with result in the next leap year",
  "be", 2563);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M12", calendar }, options).subtract(years1months2),
  2566, 2, "M02", "add 1y 2mo with result in the next year",
  "be", 2566);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2565, monthCode: "M12", calendar }, options).subtract(years1months2),
  2567, 2, "M02", "add 1y 2mo with result in the next leap year",
  "be", 2567);

TemporalHelpers.assertPlainYearMonth(
  date256303.subtract(months1n, options),
  2563, 2, "M02", "subtract 1mo from Mar in leap year",
  "be", 2563);

TemporalHelpers.assertPlainYearMonth(
  date256301.subtract(months11n, options),
  2562, 2, "M02", "subtract 11mo with result in the previous year",
  "be", 2562);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M01", calendar }, options).subtract(months11n, options),
  2563, 2, "M02", "add 11mo with result in the previous leap year",
  "be", 2563);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2565, monthCode: "M04", calendar }, options).subtract(years1months2n),
  2564, 2, "M02", "add 1y 2mo with result in the previous year",
  "be", 2564);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M04", calendar }, options).subtract(years1months2n),
  2563, 2, "M02", "add 1y 2mo with result in the previous leap year",
  "be", 2563);
