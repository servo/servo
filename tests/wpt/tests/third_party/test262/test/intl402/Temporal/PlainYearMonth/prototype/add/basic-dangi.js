// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Basic addition and subtraction in the dangi calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "dangi";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years5 = new Temporal.Duration(5);
const years5n = new Temporal.Duration(-5);

const date201802 = Temporal.PlainYearMonth.from({ year: 2018, monthCode: "M02", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date201802.add(years1),
  2019, 2, "M02", "Adding 1 year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201802.add(years5),
  2023, 2, "M02", "Adding 5 years",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201802.add(years1n),
  2017, 2, "M02", "Subtracting 1 year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201802.add(years5n),
  2013, 2, "M02", "Subtracting 5 years",
  undefined, undefined, null);

// Months

const months1 = new Temporal.Duration(0, 1);
const months1n = new Temporal.Duration(0, -1);
const months4 = new Temporal.Duration(0, 4);
const months4n = new Temporal.Duration(0, -4);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);

const date201901 = Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M01", calendar }, options);
const date201906 = Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M06", calendar }, options);
const date201911 = Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M11", calendar }, options);
const date201912 = Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M12", calendar }, options);
const date200012 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M12", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date201911.add(months1),
  2019, 12, "M12", "Adding 1 month, with result in same year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201912.add(months1),
  2020, 1, "M01", "Adding 1 month, with result in next year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201906.add(months4),
  2019, 10, "M10", "Adding 4 months, with result in same year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201912.add(months4),
  2020, 4, "M04", "Adding 4 months, with result in next year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201911.add(months1n),
  2019, 10, "M10", "Subtracting 1 month, with result in same year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201901.add(months1n),
  2018, 12, "M12", "Subtracting 1 month, with result in previous year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201906.add(months4n),
  2019, 2, "M02", "Subtracting 4 months, with result in same year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date201901.add(months4n),
  2018, 9, "M09", "Subtracting 4 months, with result in previous year",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date200012.add(months6),
  2001, 6, "M05", "Adding 6 months, with result in next year (leap year)",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  date200012.add(months6n),
  2000, 6, "M06", "Subtracting 6 months, with result in same year",
  undefined, undefined, null);
