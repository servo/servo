// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Basic addition and subtraction in the chinese calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "chinese";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years5 = new Temporal.Duration(-5);
const years5n = new Temporal.Duration(5);
const years3months6days17 = new Temporal.Duration(-3, -6, 0, -17);
const years3months6days17n = new Temporal.Duration(3, 6, 0, 17);

const date201802 = Temporal.PlainDateTime.from({ year: 2018, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }, options);
const date202302 = Temporal.PlainDateTime.from({ year: 2023, monthCode: "M02", day: 29, hour: 12, minute: 34, calendar }, options);
const date199712 = Temporal.PlainDateTime.from({ year: 1997, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date201802.subtract(years1),
  2019, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Adding 1 year to day 1 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date202302.subtract(years1),
  2024, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Adding 1 year to day 29 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date201802.subtract(years5),
  2023, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Adding 5 years to day 1 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date202302.subtract(years5),
  2028, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Adding 5 years to day 29 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date201802.subtract(years1n),
  2017, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from day 1 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date202302.subtract(years1n),
  2022, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from day 29 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date201802.subtract(years5n),
  2013, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from day 1 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date202302.subtract(years5n),
  2018, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from day 29 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date199712.subtract(years3months6days17),
  2001, 6, "M05", 18, 12, 34, 0, 0, 0, 0, "Adding 3 years/6 months/17 days to day 1 of a month"
);

TemporalHelpers.assertPlainDateTime(
  date199712.subtract(years3months6days17n),
  1994, 5, "M05", 14, 12, 34, 0, 0, 0, 0, "Subtracting 3 years/6 months/17 days from day 1 of a month"
);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months4 = new Temporal.Duration(0, -4);
const months4n = new Temporal.Duration(0, 4);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);

const date201901 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const date201906 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options);
const date201911 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M11", day: 1, hour: 12, minute: 34, calendar }, options);
const date201912 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar }, options);
const date200012 = Temporal.PlainDateTime.from({ year: 2000, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date201911.subtract(months1),
  2019, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "Adding 1 month, with result in same year"
);

TemporalHelpers.assertPlainDateTime(
  date201912.subtract(months1),
  2020, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "Adding 1 month, with result in next year"
);

TemporalHelpers.assertPlainDateTime(
  date201906.subtract(months4),
  2019, 10, "M10", 1, 12, 34, 0, 0, 0, 0, "Adding 4 months, with result in same year"
);

TemporalHelpers.assertPlainDateTime(
  date201912.subtract(months4),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "Adding 4 months, with result in next year"
);

TemporalHelpers.assertPlainDateTime(
  date201911.subtract(months1n),
  2019, 10, "M10", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month, with result in same year"
);

TemporalHelpers.assertPlainDateTime(
  date201901.subtract(months1n),
  2018, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 month, with result in previous year"
);

TemporalHelpers.assertPlainDateTime(
  date201906.subtract(months4n),
  2019, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 4 months, with result in same year"
);

TemporalHelpers.assertPlainDateTime(
  date201901.subtract(months4n),
  2018, 9, "M09", 1, 12, 34, 0, 0, 0, 0, "Subtracting 4 months, with result in previous year"
);

TemporalHelpers.assertPlainDateTime(
  date200012.subtract(months6),
  2001, 6, "M05", 1, 12, 34, 0, 0, 0, 0, "Adding 6 months, with result in next year (leap year)"
);

TemporalHelpers.assertPlainDateTime(
  date200012.subtract(months6n),
  2000, 6, "M06", 1, 12, 34, 0, 0, 0, 0, "Subtracting 6 months, with result in same year"
);

// Weeks

const months2weeks3 = new Temporal.Duration(0, /* months = */ -2, /* weeks = */ -3);
const months2weeks3n = new Temporal.Duration(0, 2, 3);
const weeks40 = new Temporal.Duration(0, 0, /* weeks = */ -40);
const weeks40n = new Temporal.Duration(0, 0, 40);

const date202101 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const date20000101 = Temporal.PlainDateTime.from({ year: 2000, month: 1, day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date202101.subtract(months2weeks3),
  2021, 3, "M03", 22, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from non-leap day/month, ending in same year"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar }, options).subtract(months2weeks3),
  2022, 3, "M03", 21, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from end of year to next year"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options).subtract(months2weeks3n),
  2021, 3, "M03", 10, 12, 34, 0, 0, 0, 0, "subtract 2 months 3 weeks from non-leap day/month, ending in same year"
);

TemporalHelpers.assertPlainDateTime(
  date202101.subtract(months2weeks3n),
  2020, 11, "M10", 10, 12, 34, 0, 0, 0, 0, "subtract 2 months 3 weeks from beginning of year to previous year"
);

TemporalHelpers.assertPlainDateTime(
  date20000101.subtract(weeks40),
  2000, 10, "M10", 16, 12, 34, 0, 0, 0, 0, "add 40 weeks, ending in same year"
);

TemporalHelpers.assertPlainDateTime(
  date20000101.subtract(weeks40n),
  1999, 3, "M03", 16, 12, 34, 0, 0, 0, 0, "subtract 40 weeks, ending in previous year"
);

// Days

const days10 = new Temporal.Duration(0, 0, 0, /* days = */ -10);
const days10n = new Temporal.Duration(0, 0, 0, 10);
const days200 = new Temporal.Duration(0, 0, 0, /* days = */ -200);
const days200n = new Temporal.Duration(0, 0, 0, 200);

const date20210129 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 29, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date202101.subtract(days10),
  2021, 1, "M01", 11, 12, 34, 0, 0, 0, 0, "add 10 days, ending in same month"
);

TemporalHelpers.assertPlainDateTime(
  date20210129.subtract(days10),
  2021, 2, "M02", 10, 12, 34, 0, 0, 0, 0, "add 10 days, ending in following month"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar }, options).subtract(days10),
  2022, 1, "M01", 10, 12, 34, 0, 0, 0, 0, "add 10 days, ending in following year"
);

TemporalHelpers.assertPlainDateTime(
  date20210129.subtract(days10n),
  2021, 1, "M01", 19, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in same month"
);

TemporalHelpers.assertPlainDateTime(
  date201906.subtract(days10n),
  2019, 5, "M05", 21, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in previous month"
);

TemporalHelpers.assertPlainDateTime(
  date202101.subtract(days10n),
  2020, 13, "M12", 21, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in previous year"
);

TemporalHelpers.assertPlainDateTime(
  date20000101.subtract(days200),
  2000, 7, "M07", 24, 12, 34, 0, 0, 0, 0, "add 200 days, ending in same year"
);

TemporalHelpers.assertPlainDateTime(
  date20000101.subtract(days200n),
  1999, 6, "M06", 8, 12, 34, 0, 0, 0, 0, "subtract 200 days, ending in previous year"
);
