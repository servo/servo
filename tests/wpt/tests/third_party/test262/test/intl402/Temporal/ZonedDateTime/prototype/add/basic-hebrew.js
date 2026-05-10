// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
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
const years3months6days17 = new Temporal.Duration(3, 6, 0, 17);
const years3months6days17n = new Temporal.Duration(-3, -6, 0, -17);

const date575712 = Temporal.ZonedDateTime.from({ year: 5757, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date577902 = Temporal.ZonedDateTime.from({ year: 5779, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date578402 = Temporal.ZonedDateTime.from({ year: 5784, monthCode: "M02", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date577902.add(years1).toPlainDateTime(),
  5780, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Adding 1 year to day 1 of a month",
  "am", 5780
);

TemporalHelpers.assertPlainDateTime(
  date578402.add(years1).toPlainDateTime(),
  5785, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Adding 1 year to day 29 of a month",
  "am", 5785
);

TemporalHelpers.assertPlainDateTime(
  date577902.add(years5).toPlainDateTime(),
  5784, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Adding 5 years to day 1 of a month",
  "am", 5784
);

TemporalHelpers.assertPlainDateTime(
  date578402.add(years5).toPlainDateTime(),
  5789, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Adding 5 years to day 29 of a month",
  "am", 5789
);

TemporalHelpers.assertPlainDateTime(
  date577902.add(years1n).toPlainDateTime(),
  5778, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from day 1 of a month",
  "am", 5778
);

TemporalHelpers.assertPlainDateTime(
  date578402.add(years1n).toPlainDateTime(),
  5783, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from day 29 of a month",
  "am", 5783
);

TemporalHelpers.assertPlainDateTime(
  date577902.add(years5n).toPlainDateTime(),
  5774, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from day 1 of a month",
  "am", 5774
);

TemporalHelpers.assertPlainDateTime(
  date578402.add(years5n).toPlainDateTime(),
  5779, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from day 29 of a month",
  "am", 5779
);

TemporalHelpers.assertPlainDateTime(
  date575712.add(years3months6days17).toPlainDateTime(),
  5761, 6, "M06", 18, 12, 34, 0, 0, 0, 0, "Adding 3y6m17d to day 1 of a month",
  "am", 5761);
var calculatedStart = date575712.add(years3months6days17).add(years3months6days17n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  5757, 13, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 3y6m17d",
  "am", 5757);

// Months

const months1 = new Temporal.Duration(0, 1);
const months1n = new Temporal.Duration(0, -1);
const months4 = new Temporal.Duration(0, 4);
const months4n = new Temporal.Duration(0, -4);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);

const date576012 = Temporal.ZonedDateTime.from({ year: 5760, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date578001 = Temporal.ZonedDateTime.from({ year: 5780, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date578006 = Temporal.ZonedDateTime.from({ year: 5780, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date578011 = Temporal.ZonedDateTime.from({ year: 5780, monthCode: "M11", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date578012 = Temporal.ZonedDateTime.from({ year: 5780, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date578011.add(months1).toPlainDateTime(),
  5780, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "Adding 1 month, with result in same year",
  "am", 5780
);

TemporalHelpers.assertPlainDateTime(
  date578012.add(months1).toPlainDateTime(),
  5781, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "Adding 1 month, with result in next year",
  "am", 5781
);

TemporalHelpers.assertPlainDateTime(
  date578006.add(months4).toPlainDateTime(),
  5780, 10, "M10", 1, 12, 34, 0, 0, 0, 0, "Adding 4 months, with result in same year",
  "am", 5780
);

TemporalHelpers.assertPlainDateTime(
  date578012.add(months4).toPlainDateTime(),
  5781, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 4 months, with result in next year",
  "am", 5781
);

TemporalHelpers.assertPlainDateTime(
  date578011.add(months1n).toPlainDateTime(),
  5780, 10, "M10", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month, with result in same year",
  "am", 5780
);

TemporalHelpers.assertPlainDateTime(
  date578001.add(months1n).toPlainDateTime(),
  5779, 13, "M12", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month, with result in previous year",
  "am", 5779
);

TemporalHelpers.assertPlainDateTime(
  date578006.add(months4n).toPlainDateTime(),
  5780, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 4 months, with result in same year",
  "am", 5780
);

TemporalHelpers.assertPlainDateTime(
  date578001.add(months4n).toPlainDateTime(),
  5779, 10, "M09", 1, 12, 34, 0, 0, 0, 0, "Subtracting 4 months, with result in previous year",
  "am", 5779
);

TemporalHelpers.assertPlainDateTime(
  date576012.add(months6).toPlainDateTime(),
  5761, 6, "M06", 1, 12, 34, 0, 0, 0, 0, "add 6 months, with result in next year",
  "am", 5761);
calculatedStart = date576012.add(months6).add(months6n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  5760, 13, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 6 months, with result in previous year",
  "am", 5760);

// Weeks

const months2weeks3 = new Temporal.Duration(0, /* months = */ 2, /* weeks = */ 3);
const months2weeks3n = new Temporal.Duration(0, -2, -3);
const weeks40 = new Temporal.Duration(0, 0, 40);
const weeks40n = new Temporal.Duration(0, 0, -40);

const date576001 = Temporal.ZonedDateTime.from({ year: 5760, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date578201 = Temporal.ZonedDateTime.from({ year: 5782, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date578201.add(months2weeks3).toPlainDateTime(),
  5782, 3, "M03", 22, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks, ending in same year",
  "am", 5782
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 5782, monthCode: "M12", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).add(months2weeks3).toPlainDateTime(),
  5783, 3, "M03", 20, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from end of year to next year",
  "am", 5783
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 5782, monthCode: "M10", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).add(months2weeks3n).toPlainDateTime(),
  5782, 8, "M07", 10, 12, 34, 0, 0, 0, 0, "subtract 2 months 3 weeks, ending in same year",
  "am", 5782
);

TemporalHelpers.assertPlainDateTime(
  date578201.add(months2weeks3n).toPlainDateTime(),
  5781, 10, "M10", 9, 12, 34, 0, 0, 0, 0, "subtract 2 months 3 weeks from beginning of year to previous year",
  "am", 5781
);

TemporalHelpers.assertPlainDateTime(
  date576001.add(weeks40).toPlainDateTime(),
  5760, 10, "M09", 14, 12, 34, 0, 0, 0, 0, "add 40 weeks, ending in same year",
  "am", 5760);
calculatedStart = date576001.add(weeks40).add(weeks40n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  5760, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 40 weeks, ending in same year",
  "am", 5760);

// Days

const days10 = new Temporal.Duration(0, 0, 0, /* days = */ 10);
const days10n = new Temporal.Duration(0, 0, 0, -10);
const days280 = new Temporal.Duration(0, 0, 0, 280);
const days280n = new Temporal.Duration(0, 0, 0, -280);

const date57800129 = Temporal.ZonedDateTime.from({ year: 5780, monthCode: "M01", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date578201.add(days10).toPlainDateTime(),
  5782, 1, "M01", 11, 12, 34, 0, 0, 0, 0, "add 10 days, ending in same month",
  "am", 5782
);

TemporalHelpers.assertPlainDateTime(
  date57800129.add(days10).toPlainDateTime(),
  5780, 2, "M02", 9, 12, 34, 0, 0, 0, 0, "add 10 days, ending in following month",
  "am", 5780
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 5782, monthCode: "M12", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).add(days10).toPlainDateTime(),
  5783, 1, "M01", 10, 12, 34, 0, 0, 0, 0, "add 10 days, ending in following year",
  "am", 5783
);

TemporalHelpers.assertPlainDateTime(
  date57800129.add(days10n).toPlainDateTime(),
  5780, 1, "M01", 19, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in same month",
  "am", 5780
);

TemporalHelpers.assertPlainDateTime(
  date578006.add(days10n).toPlainDateTime(),
  5780, 5, "M05", 21, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in previous month",
  "am", 5780
);

TemporalHelpers.assertPlainDateTime(
  date578201.add(days10n).toPlainDateTime(),
  5781, 12, "M12", 20, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in previous year",
  "am", 5781
);

TemporalHelpers.assertPlainDateTime(
  date576001.add(days280).toPlainDateTime(),
  5760, 10, "M09", 14, 12, 34, 0, 0, 0, 0, "add 280 days, ending in same year",
  "am", 5760);
calculatedStart = date576001.add(days280).add(days280n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  5760, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 280 days, ending in same year",
  "am", 5760);
