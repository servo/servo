// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Basic addition and subtraction in the islamic-umalqura calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-umalqura";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years5 = new Temporal.Duration(5);
const years5n = new Temporal.Duration(-5);
const years3months6days17 = new Temporal.Duration(3, 6, 0, 17);
const years3months6days17n = new Temporal.Duration(-3, -6, 0, -17);

const date141712 = Temporal.PlainDateTime.from({ year: 1417, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar }, options);
const date143902 = Temporal.PlainDateTime.from({ year: 1439, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }, options);
const date144402 = Temporal.PlainDateTime.from({ year: 1444, monthCode: "M02", day: 29, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date143902.add(years1),
  1440, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Adding 1 year to day 1 of a month",
  "ah", 1440
);

TemporalHelpers.assertPlainDateTime(
  date144402.add(years1),
  1445, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Adding 1 year to day 29 of a month",
  "ah", 1445
);

TemporalHelpers.assertPlainDateTime(
  date143902.add(years5),
  1444, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Adding 5 years to day 1 of a month",
  "ah", 1444
);

TemporalHelpers.assertPlainDateTime(
  date144402.add(years5),
  1449, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Adding 5 years to day 29 of a month",
  "ah", 1449
);

TemporalHelpers.assertPlainDateTime(
  date143902.add(years1n),
  1438, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from day 1 of a month",
  "ah", 1438
);

TemporalHelpers.assertPlainDateTime(
  date144402.add(years1n),
  1443, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from day 29 of a month",
  "ah", 1443
);

TemporalHelpers.assertPlainDateTime(
  date143902.add(years5n),
  1434, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from day 1 of a month",
  "ah", 1434
);

TemporalHelpers.assertPlainDateTime(
  date144402.add(years5n),
  1439, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from day 29 of a month",
  "ah", 1439
);

TemporalHelpers.assertPlainDateTime(
  date141712.add(years3months6days17),
  1421, 6, "M06", 18, 12, 34, 0, 0, 0, 0, "Adding 3y6m17d to day 1 of a month",
  "ah", 1421);
var calculatedStart = date141712.add(years3months6days17).add(years3months6days17n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1417, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 3y6m17d",
  "ah", 1417);

// Months

const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);

const date142012 = Temporal.PlainDateTime.from({ year: 1420, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar }, options);
const date1 = Temporal.PlainDateTime.from({ year: 1445, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date1.add(new Temporal.Duration(0, 8)),
  1445, 9, "M09", 1, 12, 34, 0, 0, 0, 0, "Adding 8 months to Muharram 1445 lands in Ramadan",
  "ah", 1445
);

TemporalHelpers.assertPlainDateTime(
  date1.add(new Temporal.Duration(0, 11)),
  1445, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "Adding 11 months to Muharram 1445 lands in Dhu al-Hijjah",
  "ah", 1445
);

TemporalHelpers.assertPlainDateTime(
  date1.add(new Temporal.Duration(0, 12)),
  1446, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "Adding 12 months to Muharram 1445 lands in Muharram 1446",
  "ah", 1446
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1445, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }).add(new Temporal.Duration(0, 13)),
  1446, 7, "M07", 15, 12, 34, 0, 0, 0, 0, "Adding 13 months to Jumada II 1445 lands in Rajab 1446",
  "ah", 1446
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1445, monthCode: "M03", day: 15, hour: 12, minute: 34, calendar }, options).add(new Temporal.Duration(0, 6)),
  1445, 9, "M09", 15, 12, 34, 0, 0, 0, 0, "Adding 6 months to Rabi I 1445 lands in Ramadan",
  "ah", 1445
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1444, monthCode: "M10", day: 1, hour: 12, minute: 34, calendar }).add(new Temporal.Duration(0, 5)),
  1445, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Adding 5 months to Shawwal 1444 crosses to 1445",
  "ah", 1445
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }).add(new Temporal.Duration(0, 100)),
  1408, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "Adding a large number of months",
  "ah", 1408
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1445, monthCode: "M09", day: 1, hour: 12, minute: 34, calendar }, options).add(new Temporal.Duration(0, -8)),
  1445, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "Subtracting 8 months from Ramadan 1445 lands in Muharram",
  "ah", 1445
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1445, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options).add(new Temporal.Duration(0, -12)),
  1444, 6, "M06", 1, 12, 34, 0, 0, 0, 0, "Subtracting 12 months from Jumada II 1445 lands in Jumada II 1444",
  "ah", 1444
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1445, monthCode: "M02", day: 15, hour: 12, minute: 34, calendar }, options).add(new Temporal.Duration(0, -5)),
  1444, 9, "M09", 15, 12, 34, 0, 0, 0, 0, "Subtracting 5 months from Safar 1445 crosses to Ramadan 1444",
  "ah", 1444
);

TemporalHelpers.assertPlainDateTime(
  date142012.add(months6),
  1421, 6, "M06", 1, 12, 34, 0, 0, 0, 0, "add 6 months, with result in next year",
  "ah", 1421);
calculatedStart = date142012.add(months6).add(months6n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1420, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 6 months, with result in previous year",
  "ah", 1420);

// Weeks

const months2weeks3 = new Temporal.Duration(0, /* months = */ 2, /* weeks = */ 3);
const months2weeks3n = new Temporal.Duration(0, -2, -3);
const weeks40 = new Temporal.Duration(0, 0, 40);
const weeks40n = new Temporal.Duration(0, 0, -40);

const date142001 = Temporal.PlainDateTime.from({ year: 1420, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const date144101 = Temporal.PlainDateTime.from({ year: 1441, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date144101.add(months2weeks3),
  1441, 3, "M03", 22, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks, ending in same year",
  "ah", 1441
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1441, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar }, options).add(months2weeks3),
  1442, 3, "M03", 20, 12, 34, 0, 0, 0, 0, "add 2 months 3 weeks from end of year to next year",
  "ah", 1442
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1441, monthCode: "M10", day: 1, hour: 12, minute: 34, calendar }, options).add(months2weeks3n),
  1441, 7, "M07", 9, 12, 34, 0, 0, 0, 0, "subtract 2 months 3 weeks, ending in same year",
  "ah", 1441
);

TemporalHelpers.assertPlainDateTime(
  date144101.add(months2weeks3n),
  1440, 10, "M10", 10, 12, 34, 0, 0, 0, 0, "subtract 2 months 3 weeks from beginning of year to previous year",
  "ah", 1440
);

TemporalHelpers.assertPlainDateTime(
  date142001.add(weeks40),
  1420, 10, "M10", 15, 12, 34, 0, 0, 0, 0, "add 40 weeks, ending in same year",
  "ah", 1420);
calculatedStart = date142001.add(weeks40).add(weeks40n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1420, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 40 weeks, ending in same year",
  "ah", 1420);

// Days

const days10 = new Temporal.Duration(0, 0, 0, /* days = */ 10);
const days10n = new Temporal.Duration(0, 0, 0, -10);
const days280 = new Temporal.Duration(0, 0, 0, 280);
const days280n = new Temporal.Duration(0, 0, 0, -280);

const date14390129 = Temporal.PlainDateTime.from({ year: 1439, monthCode: "M01", day: 29, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date144101.add(days10),
  1441, 1, "M01", 11, 12, 34, 0, 0, 0, 0, "add 10 days, ending in same month",
  "ah", 1441
);

TemporalHelpers.assertPlainDateTime(
  date14390129.add(days10),
  1439, 2, "M02", 9, 12, 34, 0, 0, 0, 0, "add 10 days, ending in following month",
  "ah", 1439
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1440, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar }, options).add(days10),
  1441, 1, "M01", 10, 12, 34, 0, 0, 0, 0, "add 10 days, ending in following year",
  "ah", 1441
);

TemporalHelpers.assertPlainDateTime(
  date14390129.add(days10n),
  1439, 1, "M01", 19, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in same month",
  "ah", 1439
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1439, monthCode: "M06", day: 1, hour: 12, minute: 34, calendar }, options).add(days10n),
  1439, 5, "M05", 21, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in previous month",
  "ah", 1439
);

TemporalHelpers.assertPlainDateTime(
  date144101.add(days10n),
  1440, 12, "M12", 20, 12, 34, 0, 0, 0, 0, "subtract 10 days, ending in previous year",
  "ah", 1440
);

TemporalHelpers.assertPlainDateTime(
  date142001.add(days280),
  1420, 10, "M10", 15, 12, 34, 0, 0, 0, 0, "add 280 days, ending in same year",
  "ah", 1420);
calculatedStart = date142001.add(days280).add(days280n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1420, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 40 weeks, ending in same year",
  "ah", 1420);

