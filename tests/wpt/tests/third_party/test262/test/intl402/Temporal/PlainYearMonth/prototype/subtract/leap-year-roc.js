// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Check various basic calculations involving leap years (roc calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

const date10902 = Temporal.PlainYearMonth.from({ year: 109, monthCode: "M02", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date10902.subtract(years1),
  110, 2, "M02", "add 1y in leap year",
  "roc", 110);

TemporalHelpers.assertPlainYearMonth(
  date10902.subtract(years4, options),
  113, 2, "M02", "add 4y in leap year",
  "roc", 113);

TemporalHelpers.assertPlainYearMonth(
  date10902.subtract(years1n),
  108, 2, "M02", "subtract 1y in leap year",
  "roc", 108);

TemporalHelpers.assertPlainYearMonth(
  date10902.subtract(years4n, options),
  105, 2, "M02", "subtract 4y in leap year",
  "roc", 105);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months5 = new Temporal.Duration(0, -5);
const months11n = new Temporal.Duration(0, 11);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date10901 = Temporal.PlainYearMonth.from({ year: 109, monthCode: "M01", calendar }, options);
const date10903 = Temporal.PlainYearMonth.from({ year: 109, monthCode: "M03", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date10901.subtract(months1, options),
  109, 2, "M02", "add 1mo to Jan in leap year",
  "roc", 109);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 110, monthCode: "M09", calendar }, options).subtract(months5, options),
  111, 2, "M02", "add 5mo with result in the next year",
  "roc", 111);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 108, monthCode: "M09", calendar }, options).subtract(months5, options),
  109, 2, "M02", "add 5mo with result in the next leap year",
  "roc", 109);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 110, monthCode: "M12", calendar }, options).subtract(years1months2, options),
  112, 2, "M02", "add 1y 2mo with result in the next year",
  "roc", 112);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 111, monthCode: "M12", calendar }, options).subtract(years1months2, options),
  113, 2, "M02", "add 1y 2mo with result in the next leap year",
  "roc", 113);

TemporalHelpers.assertPlainYearMonth(
  date10903.subtract(months1n, options),
  109, 2, "M02", "subtract 1mo from Mar in leap year",
  "roc", 109);

TemporalHelpers.assertPlainYearMonth(
  date10901.subtract(months11n, options),
  108, 2, "M02", "subtract 11mo with result in the previous year",
  "roc", 108);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 110, monthCode: "M01", calendar }, options).subtract(months11n, options),
  109, 2, "M02", "add 11mo with result in the previous leap year",
  "roc", 109);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 111, monthCode: "M04", calendar }, options).subtract(years1months2n, options),
  110, 2, "M02", "add 1y 2mo with result in the previous year",
  "roc", 110);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 110, monthCode: "M04", calendar }, options).subtract(years1months2n, options),
  109, 2, "M02", "add 1y 2mo with result in the previous leap year",
  "roc", 109);
