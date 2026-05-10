// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Basic addition and subtraction in the ethioaa calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethioaa";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years5 = new Temporal.Duration(-5);
const years5n = new Temporal.Duration(5);

const date750302 = Temporal.PlainYearMonth.from({ year: 7503, monthCode: "M02", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date750302.subtract(years1),
  7504, 2, "M02", "Adding 1 year", "aa", 7504, null
);

TemporalHelpers.assertPlainYearMonth(
  date750302.subtract(years5),
  7508, 2, "M02", "Adding 5 years", "aa", 7508, null
);

TemporalHelpers.assertPlainYearMonth(
  date750302.subtract(years1n),
  7502, 2, "M02", "Subtracting 1 year", "aa", 7502, null
);

TemporalHelpers.assertPlainYearMonth(
  date750302.subtract(years5n),
  7498, 2, "M02", "Subtracting 5 years", "aa", 7498, null
);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months4 = new Temporal.Duration(0, -4);
const months4n = new Temporal.Duration(0, 4);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);

const date749212 = Temporal.PlainYearMonth.from({ year: 7492, monthCode: "M12", calendar }, options);
const date750401 = Temporal.PlainYearMonth.from({ year: 7504, monthCode: "M01", calendar }, options);
const date750406 = Temporal.PlainYearMonth.from({ year: 7504, monthCode: "M06", calendar }, options);
const date750411 = Temporal.PlainYearMonth.from({ year: 7504, monthCode: "M11", calendar }, options);
const date750313 = Temporal.PlainYearMonth.from({ year: 7503, monthCode: "M13", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date750411.subtract(months1),
  7504, 12, "M12", "Adding 1 month, with result in same year", "aa", 7504, null
);

TemporalHelpers.assertPlainYearMonth(
  date750313.subtract(months1),
  7504, 1, "M01", "Adding 1 month, with result in next year", "aa", 7504, null
);

TemporalHelpers.assertPlainYearMonth(
  date750406.subtract(months4),
  7504, 10, "M10", "Adding 4 months, with result in same year", "aa", 7504, null
);

TemporalHelpers.assertPlainYearMonth(
  date750313.subtract(months4),
  7504, 4, "M04", "Adding 4 months, with result in next year", "aa", 7504, null
);

TemporalHelpers.assertPlainYearMonth(
  date750411.subtract(months1n),
  7504, 10, "M10", "Subtracting 1 month, with result in same year", "aa", 7504, null
);

TemporalHelpers.assertPlainYearMonth(
  date750401.subtract(months1n),
  7503, 13, "M13", "Subtracting 1 month, with result in previous year", "aa", 7503, null
);

TemporalHelpers.assertPlainYearMonth(
  date750406.subtract(months4n),
  7504, 2, "M02", "Subtracting 4 months, with result in same year", "aa", 7504, null
);

TemporalHelpers.assertPlainYearMonth(
  date750401.subtract(months4n),
  7503, 10, "M10", "Subtracting 4 months, with result in previous year", "aa", 7503, null
);

TemporalHelpers.assertPlainYearMonth(
  date749212.subtract(months6),
  7493, 5, "M05", "Adding 6 months, with result in next year", "aa", 7493, null
);
const calculatedStart = date749212.subtract(months6).subtract(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  7492, 12, "M12", "Subtracting 6 months, with result in previous year", "aa", 7492, null
);
