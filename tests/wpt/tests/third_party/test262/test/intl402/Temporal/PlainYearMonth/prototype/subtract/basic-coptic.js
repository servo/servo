// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Basic addition and subtraction in the coptic calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "coptic";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years5 = new Temporal.Duration(-5);
const years5n = new Temporal.Duration(5);

const date174202 = Temporal.PlainYearMonth.from({ year: 1742, monthCode: "M02", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date174202.subtract(years1),
  1743, 2, "M02", "Adding 1 year", "am", 1743, null
);

TemporalHelpers.assertPlainYearMonth(
  date174202.subtract(years5),
  1747, 2, "M02", "Adding 5 years", "am", 1747, null
);

TemporalHelpers.assertPlainYearMonth(
  date174202.subtract(years1n),
  1741, 2, "M02", "Subtracting 1 year", "am", 1741, null
);

TemporalHelpers.assertPlainYearMonth(
  date174202.subtract(years5n),
  1737, 2, "M02", "Subtracting 5 years", "am", 1737, null
);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months4 = new Temporal.Duration(0, -4);
const months4n = new Temporal.Duration(0, 4);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);

const date171612 = Temporal.PlainYearMonth.from({ year: 1716, monthCode: "M12", calendar }, options);
const date174301 = Temporal.PlainYearMonth.from({ year: 1743, monthCode: "M01", calendar }, options);
const date174306 = Temporal.PlainYearMonth.from({ year: 1743, monthCode: "M06", calendar }, options);
const date174311 = Temporal.PlainYearMonth.from({ year: 1743, monthCode: "M11", calendar }, options);
const date174213 = Temporal.PlainYearMonth.from({ year: 1742, monthCode: "M13", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date174311.subtract(months1),
  1743, 12, "M12", "Adding 1 month, with result in same year", "am", 1743, null
);

TemporalHelpers.assertPlainYearMonth(
  date174213.subtract(months1),
  1743, 1, "M01", "Adding 1 month, with result in next year", "am", 1743, null
);

TemporalHelpers.assertPlainYearMonth(
  date174306.subtract(months4),
  1743, 10, "M10", "Adding 4 months, with result in same year", "am", 1743, null
);

TemporalHelpers.assertPlainYearMonth(
  date174213.subtract(months4),
  1743, 4, "M04", "Adding 4 months, with result in next year", "am", 1743, null
);

TemporalHelpers.assertPlainYearMonth(
  date174311.subtract(months1n),
  1743, 10, "M10", "Subtracting 1 month, with result in same year", "am", 1743, null
);

TemporalHelpers.assertPlainYearMonth(
  date174301.subtract(months1n),
  1742, 13, "M13", "Subtracting 1 month, with result in previous year", "am", 1742, null
);

TemporalHelpers.assertPlainYearMonth(
  date174306.subtract(months4n),
  1743, 2, "M02", "Subtracting 4 months, with result in same year", "am", 1743, null
);

TemporalHelpers.assertPlainYearMonth(
  date174301.subtract(months4n),
  1742, 10, "M10", "Subtracting 4 months, with result in previous year", "am", 1742, null
);

TemporalHelpers.assertPlainYearMonth(
  date171612.subtract(months6),
  1717, 5, "M05", "Adding 6 months, with result in next year", "am", 1717, null
);
const calculatedStart = date171612.subtract(months6).subtract(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  1716, 12, "M12", "Subtracting 6 months, with result in previous year", "am", 1716, null
);
