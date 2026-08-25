// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Basic addition and subtraction in the hebrew calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years5 = new Temporal.Duration(5);
const years5n = new Temporal.Duration(-5);

const date577902 = Temporal.PlainYearMonth.from({ year: 5779, monthCode: "M02", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date577902.add(years1),
  5780, 2, "M02", "Adding 1 year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date577902.add(years5),
  5784, 2, "M02", "Adding 5 years",
  "am", 5784, null
);

TemporalHelpers.assertPlainYearMonth(
  date577902.add(years1n),
  5778, 2, "M02", "Subtracting 1 year",
  "am", 5778, null
);

TemporalHelpers.assertPlainYearMonth(
  date577902.add(years5n),
  5774, 2, "M02", "Subtracting 5 years",
  "am", 5774, null
);

// Months

const months1 = new Temporal.Duration(0, 1);
const months1n = new Temporal.Duration(0, -1);
const months4 = new Temporal.Duration(0, 4);
const months4n = new Temporal.Duration(0, -4);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);

const date576012 = Temporal.PlainYearMonth.from({ year: 5760, monthCode: "M12", calendar }, options);
const date578001 = Temporal.PlainYearMonth.from({ year: 5780, monthCode: "M01", calendar }, options);
const date578006 = Temporal.PlainYearMonth.from({ year: 5780, monthCode: "M06", calendar }, options);
const date578011 = Temporal.PlainYearMonth.from({ year: 5780, monthCode: "M11", calendar }, options);
const date578012 = Temporal.PlainYearMonth.from({ year: 5780, monthCode: "M12", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date578011.add(months1),
  5780, 12, "M12", "Adding 1 month, with result in same year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578012.add(months1),
  5781, 1, "M01", "Adding 1 month, with result in next year",
  "am", 5781, null
);

TemporalHelpers.assertPlainYearMonth(
  date578006.add(months4),
  5780, 10, "M10", "Adding 4 months, with result in same year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578012.add(months4),
  5781, 4, "M04", "Adding 4 months, with result in next year",
  "am", 5781, null
);

TemporalHelpers.assertPlainYearMonth(
  date578011.add(months1n),
  5780, 10, "M10", "Subtracting 1 month, with result in same year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578001.add(months1n),
  5779, 13, "M12", "Subtracting 1 month, with result in previous year",
  "am", 5779, null
);

TemporalHelpers.assertPlainYearMonth(
  date578006.add(months4n),
  5780, 2, "M02", "Subtracting 4 months, with result in same year",
  "am", 5780, null
);

TemporalHelpers.assertPlainYearMonth(
  date578001.add(months4n),
  5779, 10, "M09", "Subtracting 4 months, with result in previous year",
  "am", 5779, null
);

TemporalHelpers.assertPlainYearMonth(
  date576012.add(months6),
  5761, 6, "M06", "add 6 months, with result in next year",
  "am", 5761, null);
const calculatedStart = date576012.add(months6).add(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  5760, 13, "M12", "subtract 6 months, with result in previous year",
  "am", 5760, null);
