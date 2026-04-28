// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Basic addition and subtraction in the islamic-civil calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-civil";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years5 = new Temporal.Duration(5);
const years5n = new Temporal.Duration(-5);

const date143902 = Temporal.PlainYearMonth.from({ year: 1439, monthCode: "M02", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date143902.add(years1),
  1440, 2, "M02", "Adding 1 year",
  "ah", 1440, null
);

TemporalHelpers.assertPlainYearMonth(
  date143902.add(years5),
  1444, 2, "M02", "Adding 5 years",
  "ah", 1444, null
);

TemporalHelpers.assertPlainYearMonth(
  date143902.add(years1n),
  1438, 2, "M02", "Subtracting 1 year",
  "ah", 1438, null
);

TemporalHelpers.assertPlainYearMonth(
  date143902.add(years5n),
  1434, 2, "M02", "Subtracting 5 years",
  "ah", 1434, null
);

// Months

const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);

const date142012 = Temporal.PlainYearMonth.from({ year: 1420, monthCode: "M12", calendar }, options);
const date144501 = Temporal.PlainYearMonth.from({ year: 1445, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  date144501.add(new Temporal.Duration(0, 8)),
  1445, 9, "M09", "Adding 8 months to Muharram 1445 lands in Ramadan",
  "ah", 1445, null
);

TemporalHelpers.assertPlainYearMonth(
  date144501.add(new Temporal.Duration(0, 11)),
  1445, 12, "M12", "Adding 11 months to Muharram 1445 lands in Dhu al-Hijjah",
  "ah", 1445, null
);

TemporalHelpers.assertPlainYearMonth(
  date144501.add(new Temporal.Duration(0, 12)),
  1446, 1, "M01", "Adding 12 months to Muharram 1445 lands in Muharram 1446",
  "ah", 1446, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1445, monthCode: "M06", calendar }).add(new Temporal.Duration(0, 13)),
  1446, 7, "M07", "Adding 13 months to Jumada II 1445 lands in Rajab 1446",
  "ah", 1446, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1445, monthCode: "M03", calendar }, options).add(new Temporal.Duration(0, 6)),
  1445, 9, "M09", "Adding 6 months to Rabi I 1445 lands in Ramadan",
  "ah", 1445, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1444, monthCode: "M10", calendar }).add(new Temporal.Duration(0, 5)),
  1445, 3, "M03", "Adding 5 months to Shawwal 1444 crosses to 1445",
  "ah", 1445, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1400, monthCode: "M01", calendar }).add(new Temporal.Duration(0, 100)),
  1408, 5, "M05", "Adding a large number of months",
  "ah", 1408, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1445, monthCode: "M09", calendar }, options).add(new Temporal.Duration(0, -8)),
  1445, 1, "M01", "Subtracting 8 months from Ramadan 1445 lands in Muharram",
  "ah", 1445, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1445, monthCode: "M06", calendar }, options).add(new Temporal.Duration(0, -12)),
  1444, 6, "M06", "Subtracting 12 months from Jumada II 1445 lands in Jumada II 1444",
  "ah", 1444, null
);

TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1445, monthCode: "M02", calendar }, options).add(new Temporal.Duration(0, -5)),
  1444, 9, "M09", "Subtracting 5 months from Safar 1445 crosses to Ramadan 1444",
  "ah", 1444, null
);

TemporalHelpers.assertPlainYearMonth(
  date142012.add(months6),
  1421, 6, "M06", "add 6 months, with result in next year",
  "ah", 1421, null);
const calculatedStart = date142012.add(months6).add(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  1420, 12, "M12", "subtract 6 months, with result in previous year",
  "ah", 1420, null);
