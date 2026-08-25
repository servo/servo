// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Basic addition and subtraction in the ethiopic calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years5 = new Temporal.Duration(-5);
const years5n = new Temporal.Duration(5);
const years3months6days17 = new Temporal.Duration(-3, -6, 0, -17);
const years3months6days17n = new Temporal.Duration(3, 6, 0, 17);

const date199712 = Temporal.PlainDate.from({ year: 1997, monthCode: "M12", day: 1, calendar }, options);
const date201802 = Temporal.PlainDate.from({ year: 2018, monthCode: "M02", day: 1, calendar }, options);
const date202302 = Temporal.PlainDate.from({ year: 2023, monthCode: "M02", day: 29, calendar }, options);

TemporalHelpers.assertPlainDate(
  date201802.subtract(years1),
  2019, 2, "M02", 1, "Adding 1 year to day 1 of a month", "am", 2019
);

TemporalHelpers.assertPlainDate(
  date202302.subtract(years1),
  2024, 2, "M02", 29, "Adding 1 year to day 29 of a month", "am", 2024
);

TemporalHelpers.assertPlainDate(
  date201802.subtract(years5),
  2023, 2, "M02", 1, "Adding 5 years to day 1 of a month", "am", 2023
);

TemporalHelpers.assertPlainDate(
  date202302.subtract(years5),
  2028, 2, "M02", 29, "Adding 5 years to day 29 of a month", "am", 2028
);

TemporalHelpers.assertPlainDate(
  date201802.subtract(years1n),
  2017, 2, "M02", 1, "Subtracting 1 year from day 1 of a month", "am", 2017
);

TemporalHelpers.assertPlainDate(
  date202302.subtract(years1n),
  2022, 2, "M02", 29, "Subtracting 1 year from day 29 of a month", "am", 2022
);

TemporalHelpers.assertPlainDate(
  date201802.subtract(years5n),
  2013, 2, "M02", 1, "Subtracting 5 years from day 1 of a month", "am", 2013
);

TemporalHelpers.assertPlainDate(
  date202302.subtract(years5n),
  2018, 2, "M02", 29, "Subtracting 5 years from day 29 of a month", "am", 2018
);

TemporalHelpers.assertPlainDate(
  date199712.subtract(years3months6days17),
  2001, 5, "M05", 18, "Adding 3 years, 6 months and 17 days to day 1 of a month", "am", 2001
);
var calculatedStart = date199712.subtract(years3months6days17).subtract(years3months6days17n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  1997, 12, "M12", 1, "Subtracting 3 years, 6 months and 17 days from day 18 of a month", "am", 1997
);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months4 = new Temporal.Duration(0, -4);
const months4n = new Temporal.Duration(0, 4);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);

const date200012 = Temporal.PlainDate.from({ year: 2000, monthCode: "M12", day: 1, calendar }, options);
const date201901 = Temporal.PlainDate.from({ year: 2019, monthCode: "M01", day: 1, calendar }, options);
const date201906 = Temporal.PlainDate.from({ year: 2019, monthCode: "M06", day: 1, calendar }, options);
const date201911 = Temporal.PlainDate.from({ year: 2019, monthCode: "M11", day: 1, calendar }, options);
const date201813 = Temporal.PlainDate.from({ year: 2018, monthCode: "M13", day: 1, calendar }, options);

TemporalHelpers.assertPlainDate(
  date201911.subtract(months1),
  2019, 12, "M12", 1, "Adding 1 month, with result in same year", "am", 2019
);

TemporalHelpers.assertPlainDate(
  date201813.subtract(months1),
  2019, 1, "M01", 1, "Adding 1 month, with result in next year", "am", 2019
);

TemporalHelpers.assertPlainDate(
  date201906.subtract(months4),
  2019, 10, "M10", 1, "Adding 4 months, with result in same year", "am", 2019
);

TemporalHelpers.assertPlainDate(
  date201813.subtract(months4),
  2019, 4, "M04", 1, "Adding 4 months, with result in next year", "am", 2019
);

TemporalHelpers.assertPlainDate(
  date201911.subtract(months1n),
  2019, 10, "M10", 1, "Subtracting 1 month, with result in same year", "am", 2019
);

TemporalHelpers.assertPlainDate(
  date201901.subtract(months1n),
  2018, 13, "M13", 1, "Subtracting 1 month, with result in previous year", "am", 2018
);

TemporalHelpers.assertPlainDate(
  date201906.subtract(months4n),
  2019, 2, "M02", 1, "Subtracting 4 months, with result in same year", "am", 2019
);

TemporalHelpers.assertPlainDate(
  date201901.subtract(months4n),
  2018, 10, "M10", 1, "Subtracting 4 months, with result in previous year", "am", 2018
);

TemporalHelpers.assertPlainDate(
  date200012.subtract(months6),
  2001, 5, "M05", 1, "Adding 6 months, with result in next year", "am", 2001
);
calculatedStart = date200012.subtract(months6).subtract(months6n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  2000, 12, "M12", 1, "Subtracting 6 months, with result in previous year", "am", 2000
);

// Weeks

const months2weeks3 = new Temporal.Duration(0, /* months = */ -2, /* weeks = */ -3);
const months2weeks3n = new Temporal.Duration(0, 2, 3);
const weeks40 = new Temporal.Duration(0, 0, -40);
const weeks40n = new Temporal.Duration(0, 0, 40);

const date200001 = Temporal.PlainDate.from({ year: 2000, monthCode: "M01", day: 1, calendar }, options);
const date202101 = Temporal.PlainDate.from({ year: 2021, monthCode: "M01", day: 1, calendar }, options);

TemporalHelpers.assertPlainDate(
  date202101.subtract(months2weeks3),
  2021, 3, "M03", 22, "add 2 months 3 weeks from non-leap day/month, ending in same year", "am", 2021
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M12", day: 29, calendar }, options).subtract(months2weeks3),
  2022, 2, "M02", 20, "add 2 months 3 weeks from end of year to next year", "am", 2022
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M06", day: 1, calendar }, options).subtract(months2weeks3n),
  2021, 3, "M03", 10, "subtract 2 months 3 weeks from non-leap day/month, ending in same year", "am", 2021
);

TemporalHelpers.assertPlainDate(
  date202101.subtract(months2weeks3n),
  2020, 11, "M11", 10, "subtract 2 months 3 weeks from beginning of year to previous year", "am", 2020
);

TemporalHelpers.assertPlainDate(
  date200001.subtract(weeks40),
  2000, 10, "M10", 11, "Adding 40 weeks, with result in same year", "am", 2000
);
calculatedStart = date200001.subtract(weeks40).subtract(weeks40n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  2000, 1, "M01", 1, "Subtracting 40 weeks, with result in same year", "am", 2000
);

// Days

const days10 = new Temporal.Duration(0, 0, 0, /* days = */ -10);
const days10n = new Temporal.Duration(0, 0, 0, 10);
const days280 = new Temporal.Duration(0, 0, 0, /* days = */ -280);
const days280n = new Temporal.Duration(0, 0, 0, 280);

const date20210129 = Temporal.PlainDate.from({ year: 2021, monthCode: "M01", day: 30, calendar }, options);

TemporalHelpers.assertPlainDate(
  date202101.subtract(days10),
  2021, 1, "M01", 11, "add 10 days, ending in same month", "am", 2021
);

TemporalHelpers.assertPlainDate(
  date20210129.subtract(days10),
  2021, 2, "M02", 10, "add 10 days, ending in following month", "am", 2021
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M13", day: 5, calendar }, options).subtract(days10),
  2022, 1, "M01", 10, "add 10 days, ending in following year", "am", 2022
);

TemporalHelpers.assertPlainDate(
  date20210129.subtract(days10n),
  2021, 1, "M01", 20, "subtract 10 days, ending in same month", "am", 2021
);

TemporalHelpers.assertPlainDate(
  date201906.subtract(days10n),
  2019, 5, "M05", 21, "subtract 10 days, ending in previous month", "am", 2019
);

TemporalHelpers.assertPlainDate(
  date202101.subtract(days10n),
  2020, 12, "M12", 26, "subtract 10 days, ending in previous year", "am", 2020
);

TemporalHelpers.assertPlainDate(
  date200001.subtract(days280),
  2000, 10, "M10", 11, "Adding 280 days, with result in same year", "am", 2000
);
calculatedStart = date200001.subtract(days280).subtract(weeks40n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  2000, 1, "M01", 1, "Subtracting 280 days, with result in same year", "am", 2000
);
