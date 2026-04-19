// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
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
const years3months6days17 = new Temporal.Duration(-3, -6, 0, -17);
const years3months6days17n = new Temporal.Duration(3, 6, 0, 17);

const date171312 = Temporal.ZonedDateTime.from({ year: 1713, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date174202 = Temporal.ZonedDateTime.from({ year: 1742, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date174702 = Temporal.ZonedDateTime.from({ year: 1747, monthCode: "M02", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date174202.subtract(years1).toPlainDateTime(),
  1743, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Adding 1 year to day 1 of a month", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date174702.subtract(years1).toPlainDateTime(),
  1748, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Adding 1 year to day 29 of a month", "am", 1748
);

TemporalHelpers.assertPlainDateTime(
  date174202.subtract(years5).toPlainDateTime(),
  1747, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Adding 5 years to day 1 of a month", "am", 1747
);

TemporalHelpers.assertPlainDateTime(
  date174702.subtract(years5).toPlainDateTime(),
  1752, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Adding 5 years to day 29 of a month", "am", 1752
);

TemporalHelpers.assertPlainDateTime(
  date174202.subtract(years1n).toPlainDateTime(),
  1741, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from day 1 of a month", "am", 1741
);

TemporalHelpers.assertPlainDateTime(
  date174702.subtract(years1n).toPlainDateTime(),
  1746, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from day 29 of a month", "am", 1746
);

TemporalHelpers.assertPlainDateTime(
  date174202.subtract(years5n).toPlainDateTime(),
  1737, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from day 1 of a month", "am", 1737
);

TemporalHelpers.assertPlainDateTime(
  date174702.subtract(years5n).toPlainDateTime(),
  1742, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from day 29 of a month", "am", 1742
);

TemporalHelpers.assertPlainDateTime(
  date171312.subtract(years3months6days17).toPlainDateTime(),
  1717, 5, "M05", 18, 12, 34, 0, 0, 0, 0, "Adding 3 years, 6 months and 17 days to day 1 of a month", "am", 1717
);
var calculatedStart = date171312.subtract(years3months6days17).subtract(years3months6days17n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  1713, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "Subtracting 3 years, 6 months and 17 days from day 18 of a month", "am", 1713
);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months4 = new Temporal.Duration(0, -4);
const months4n = new Temporal.Duration(0, 4);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);

const date171612 = Temporal.ZonedDateTime.from({ year: 1716, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date174301 = Temporal.ZonedDateTime.from({ year: 1743, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date174306 = Temporal.ZonedDateTime.from({ year: 1743, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date174311 = Temporal.ZonedDateTime.from({ year: 1743, monthCode: "M11", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date174213 = Temporal.ZonedDateTime.from({ year: 1742, monthCode: "M13", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date174311.subtract(months1).toPlainDateTime(),
  1743, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "Adding 1 month, with result in same year", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date174213.subtract(months1).toPlainDateTime(),
  1743, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "Adding 1 month, with result in next year", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date174306.subtract(months4).toPlainDateTime(),
  1743, 10, "M10", 1, 12, 34, 0, 0, 0, 0, "Adding 4 months, with result in same year", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date174213.subtract(months4).toPlainDateTime(),
  1743, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 4 months, with result in next year", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date174311.subtract(months1n).toPlainDateTime(),
  1743, 10, "M10", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month, with result in same year", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date174301.subtract(months1n).toPlainDateTime(),
  1742, 13, "M13", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month, with result in previous year", "am", 1742
);

TemporalHelpers.assertPlainDateTime(
  date174306.subtract(months4n).toPlainDateTime(),
  1743, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 4 months, with result in same year", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date174301.subtract(months4n).toPlainDateTime(),
  1742, 10, "M10", 1, 12, 34, 0, 0, 0, 0, "Subtracting 4 months, with result in previous year", "am", 1742
);

TemporalHelpers.assertPlainDateTime(
  date171612.subtract(months6).toPlainDateTime(),
  1717, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "Adding 6 months, with result in next year", "am", 1717
);
calculatedStart = date171612.subtract(months6).subtract(months6n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  1716, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "Subtracting 6 months, with result in previous year", "am", 1716
);

// Weeks

const months2weeks3 = new Temporal.Duration(0, /* months = */ -2, /* weeks = */ -3);
const months2weeks3n = new Temporal.Duration(0, 2, 3);
const weeks40 = new Temporal.Duration(0, 0, -40);
const weeks40n = new Temporal.Duration(0, 0, 40);

const date171601 = Temporal.ZonedDateTime.from({ year: 1716, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date174401 = Temporal.ZonedDateTime.from({ year: 1744, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date174401.subtract(months2weeks3).toPlainDateTime(),
  1744, 3, "M03", 22, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from non-leap day/month, ending in same year", "am", 1744
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 1744, monthCode: "M12", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(months2weeks3).toPlainDateTime(),
  1745, 2, "M02", 20, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from end of year to next year", "am", 1745
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 1744, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(months2weeks3n).toPlainDateTime(),
  1744, 3, "M03", 10, 12, 34, 0, 0, 0, 0, "subtract 2 months 3 weeks from non-leap day/month, ending in same year", "am", 1744
);

TemporalHelpers.assertPlainDateTime(
  date174401.subtract(months2weeks3n).toPlainDateTime(),
  1743, 11, "M11", 10, 12, 34, 0, 0, 0, 0, "subtract 2 months 3 weeks from beginning of year to previous year", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date171601.subtract(weeks40).toPlainDateTime(),
  1716, 10, "M10", 11, 12, 34, 0, 0, 0, 0, "Adding 40 weeks, with result in same year", "am", 1716
);
calculatedStart = date171601.subtract(weeks40).subtract(weeks40n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  1716, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "Subtracting 40 weeks, with result in same year", "am", 1716
);

// Days

const days10 = new Temporal.Duration(0, 0, 0, /* days = */ -10);
const days10n = new Temporal.Duration(0, 0, 0, 10);
const days280 = new Temporal.Duration(0, 0, 0, /* days = */ -280);
const days280n = new Temporal.Duration(0, 0, 0, 280);

const date17440129 = Temporal.ZonedDateTime.from({ year: 1744, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date174401.subtract(days10).toPlainDateTime(),
  1744, 1, "M01", 11, 12, 34, 0, 0, 0, 0, "add 10 days, ending in same month", "am", 1744
);

TemporalHelpers.assertPlainDateTime(
  date17440129.subtract(days10).toPlainDateTime(),
  1744, 2, "M02", 10, 12, 34, 0, 0, 0, 0, "add 10 days, ending in following month", "am", 1744
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 1744, monthCode: "M13", day: 5, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(days10).toPlainDateTime(),
  1745, 1, "M01", 10, 12, 34, 0, 0, 0, 0, "add 10 days, ending in following year", "am", 1745
);

TemporalHelpers.assertPlainDateTime(
  date17440129.subtract(days10n).toPlainDateTime(),
  1744, 1, "M01", 20, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in same month", "am", 1744
);

TemporalHelpers.assertPlainDateTime(
  date174306.subtract(days10n).toPlainDateTime(),
  1743, 5, "M05", 21, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in previous month", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date174401.subtract(days10n).toPlainDateTime(),
  1743, 12, "M12", 27, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in previous year", "am", 1743
);

TemporalHelpers.assertPlainDateTime(
  date171601.subtract(days280).toPlainDateTime(),
  1716, 10, "M10", 11, 12, 34, 0, 0, 0, 0, "Adding 280 days, with result in same year", "am", 1716
);
calculatedStart = date171601.subtract(days280).subtract(weeks40n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  1716, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "Subtracting 280 days, with result in same year", "am", 1716
);
