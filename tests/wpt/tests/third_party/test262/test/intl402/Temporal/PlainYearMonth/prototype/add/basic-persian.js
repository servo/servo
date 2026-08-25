// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: >
  Check various basic calculations not involving leap years or constraining
  (persian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "persian";

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

const date140007 = Temporal.PlainYearMonth.from({ year: 1400, monthCode: "M07", calendar });

TemporalHelpers.assertPlainYearMonth(
  date140007.add(years1),
  1401, 7, "M07", "add 1y",
  "ap", 1401, null);
TemporalHelpers.assertPlainYearMonth(
  date140007.add(years4),
  1404, 7, "M07", "add 4y",
  "ap", 1404, null);

TemporalHelpers.assertPlainYearMonth(
  date140007.add(years1n),
  1399, 7, "M07", "subtract 1y",
  "ap", 1399, null);
TemporalHelpers.assertPlainYearMonth(
  date140007.add(years4n),
  1396, 7, "M07", "subtract 4y",
  "ap", 1396, null);

// Months

const months5 = new Temporal.Duration(0, 5);
const months5n = new Temporal.Duration(0, -5);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);
const months8 = new Temporal.Duration(0, 8);
const months8n = new Temporal.Duration(0, -8);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

const date137812 = Temporal.PlainYearMonth.from({ year: 1378, monthCode: "M12", calendar });

TemporalHelpers.assertPlainYearMonth(
  date140007.add(months5),
  1400, 12, "M12", "add 5mo with result in the same year",
  "ap", 1400, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1400, monthCode: "M08", calendar }).add(months5),
  1401, 1, "M01", "add 5mo with result in the next year",
  "ap", 1401, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1398, monthCode: "M10", calendar }).add(months5),
  1399, 3, "M03", "add 5mo with result in the next year on day 1 of month",
  "ap", 1399, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1400, monthCode: "M06", calendar }).add(months8),
  1401, 2, "M02", "add 8mo with result in the next year on day 31 of month",
  "ap", 1401, null);

TemporalHelpers.assertPlainYearMonth(
  date140007.add(years1months2),
  1401, 9, "M09", "add 1y 2mo",
  "ap", 1401, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1400, monthCode: "M11", calendar }).add(years1months2),
  1402, 1, "M01", "add 1y 2mo with result in the next year",
  "ap", 1402, null);

TemporalHelpers.assertPlainYearMonth(
  date140007.add(months5n),
  1400, 2, "M02", "subtract 5mo with result in the same year",
  "ap", 1400, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1400, monthCode: "M01", calendar }).add(months5n),
  1399, 8, "M08", "subtract 5mo with result in the previous year",
  "ap", 1399, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1398, monthCode: "M02", calendar }).add(months5n),
  1397, 9, "M09", "subtract 5mo with result in the previous year on day 1 of month",
  "ap", 1397, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1400, monthCode: "M02", calendar }).add(months8n),
  1399, 6, "M06", "subtract 8mo with result in the previous year on day 31 of month",
  "ap", 1399, null);

TemporalHelpers.assertPlainYearMonth(
  date140007.add(years1months2n),
  1399, 5, "M05", "subtract 1y 2mo",
  "ap", 1399, null);
TemporalHelpers.assertPlainYearMonth(
  Temporal.PlainYearMonth.from({ year: 1400, monthCode: "M02", calendar }).add(years1months2n),
  1398, 12, "M12", "subtract 1y 2mo with result in the previous year",
  "ap", 1398, null);

TemporalHelpers.assertPlainYearMonth(
  date137812.add(months6),
  1379, 6, "M06", "add 6 months, with result in next year",
  "ap", 1379, null);
const calculatedStart = date137812.add(months6).add(months6n);
TemporalHelpers.assertPlainYearMonth(
  calculatedStart,
  1378, 12, "M12", "subtract 6 months, with result in previous year",
  "ap", 1378, null);
