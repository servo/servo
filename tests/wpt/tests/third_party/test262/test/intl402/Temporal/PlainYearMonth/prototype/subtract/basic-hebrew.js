// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Basic addition and subtraction in the hebrew calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years5 = new Temporal.Duration(-5);
const years5n = new Temporal.Duration(5);

const date577902 = Temporal.PlainYearMonth.from({ year: 5779, monthCode: "M02", calendar }, options);
const date578402 = Temporal.PlainYearMonth.from({ year: 5784, monthCode: "M02", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date577902.subtract(years1),
  5780, 2, "M02", "Adding 1 year to day 1 of a month",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578402.subtract(years1),
  5785, 2, "M02", "Adding 1 year to day 29 of a month",
  "am", 5785, null
);

TemporalHelpers.assertPlainYearMonth(
  date577902.subtract(years5),
  5784, 2, "M02", "Adding 5 years to day 1 of a month",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  date578402.subtract(years5),
  5789, 2, "M02", "Adding 5 years to day 29 of a month",
  "am", 5789, null
);

TemporalHelpers.assertPlainYearMonth(
  date577902.subtract(years1n),
  5778, 2, "M02", "Subtracting 1 year from day 1 of a month",
  "am", 5778, null
);

TemporalHelpers.assertPlainYearMonth(
  date578402.subtract(years1n),
  5783, 2, "M02", "Subtracting 1 year from day 29 of a month",
  "am", 5783, null
);

TemporalHelpers.assertPlainYearMonth(
  date577902.subtract(years5n),
  5774, 2, "M02", "Subtracting 5 years from day 1 of a month",
  "am", 5774, null
);

TemporalHelpers.assertPlainYearMonth(
  date578402.subtract(years5n),
  5779, 2, "M02", "Subtracting 5 years from day 29 of a month",
  "am", 5779, null
);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months4 = new Temporal.Duration(0, -4);
const months4n = new Temporal.Duration(0, 4);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);

const date576012 = Temporal.PlainYearMonth.from({ year: 5760, monthCode: "M12", calendar }, options);
const date578001 = Temporal.PlainYearMonth.from({ year: 5780, monthCode: "M01", calendar }, options);
const date578006 = Temporal.PlainYearMonth.from({ year: 5780, monthCode: "M06", calendar }, options);
const date578011 = Temporal.PlainYearMonth.from({ year: 5780, monthCode: "M11", calendar }, options);
const date578012 = Temporal.PlainYearMonth.from({ year: 5780, monthCode: "M12", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date578011.subtract(months1),
  5780, 12, "M12", "Adding 1 month, with result in same year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578012.subtract(months1),
  5781, 1, "M01", "Adding 1 month, with result in next year",
  "am", 5781, null
);

TemporalHelpers.assertPlainYearMonth(
  date578006.subtract(months4),
  5780, 10, "M10", "Adding 4 months, with result in same year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578012.subtract(months4),
  5781, 4, "M04", "Adding 4 months, with result in next year",
  "am", 5781, null
);

TemporalHelpers.assertPlainYearMonth(
  date578011.subtract(months1n),
  5780, 10, "M10", "Subtracting 1 month, with result in same year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578001.subtract(months1n),
  5779, 13, "M12", "Subtracting 1 month, with result in previous year",
  "am", 5779, null
);

TemporalHelpers.assertPlainYearMonth(
  date578006.subtract(months4n),
  5780, 2, "M02", "Subtracting 4 months, with result in same year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578001.subtract(months4n),
  5779, 10, "M09", "Subtracting 4 months, with result in previous year",
  "am", 5779, null
);

TemporalHelpers.assertPlainYearMonth(
  date576012.subtract(months6),
  5761, 6, "M06", "add 6 months, with result in next year",
  "am", 5761, null);
const calculatedStart = date576012.subtract(months6).subtract(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  5760, 13, "M12", "subtract 6 months, with result in previous year",
  "am", 5760, null);
