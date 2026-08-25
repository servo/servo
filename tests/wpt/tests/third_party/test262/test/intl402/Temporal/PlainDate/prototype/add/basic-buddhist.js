// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: >
  Check various basic calculations not involving leap years or constraining
  (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);
const years3months6days17 = new Temporal.Duration(3, 6, 0, 17);
const years3months6days17n = new Temporal.Duration(-3, -6, 0, -17);

const date25551201 = Temporal.PlainDate.from({ year: 2555, monthCode: "M12", day: 1, calendar });
const date25640716 = Temporal.PlainDate.from({ year: 2564, monthCode: "M07", day: 16, calendar });

TemporalHelpers.assertPlainDate(
  date25640716.add(years1),
  2565, 7, "M07", 16, "add 1y",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  date25640716.add(years4),
  2568, 7, "M07", 16, "add 4y",
  "be", 2568);

TemporalHelpers.assertPlainDate(
  date25640716.add(years1n),
  2563, 7, "M07", 16, "subtract 1y",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25640716.add(years4n),
  2560, 7, "M07", 16, "subtract 4y",
  "be", 2560);

TemporalHelpers.assertPlainDate(
  date25551201.add(years3months6days17),
  2559, 6, "M06", 18, "add 3y6m17d",
  "be", 2559);
var calculatedStart = date25551201.add(years3months6days17).add(years3months6days17n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  2555, 12, "M12", 1, "subtract 3y6m17d",
  "be", 2555);

// Months

const months5 = new Temporal.Duration(0, 5);
const months5n = new Temporal.Duration(0, -5);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

TemporalHelpers.assertPlainDate(
  date25640716.add(months5),
  2564, 12, "M12", 16, "add 5mo with result in the same year",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M08", day: 16, calendar }).add(months5),
  2565, 1, "M01", 16, "add 5mo with result in the next year",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2562, monthCode: "M10", day: 1, calendar }).add(months5),
  2563, 3, "M03", 1, "add 5mo with result in the next year on day 1 of month",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M10", day: 31, calendar }).add(months5),
  2565, 3, "M03", 31, "add 5mo with result in the next year on day 31 of month",
  "be", 2565);

TemporalHelpers.assertPlainDate(
  date25640716.add(years1months2),
  2565, 9, "M09", 16, "add 1y 2mo",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M11", day: 30, calendar }).add(years1months2),
  2566, 1, "M01", 30, "add 1y 2mo with result in the next year",
  "be", 2566);

TemporalHelpers.assertPlainDate(
  date25640716.add(months5n),
  2564, 2, "M02", 16, "subtract 5mo with result in the same year",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M01", day: 16, calendar }).add(months5n),
  2563, 8, "M08", 16, "subtract 5mo with result in the previous year",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2562, monthCode: "M02", day: 1, calendar }).add(months5n),
  2561, 9, "M09", 1, "subtract 5mo with result in the previous year on day 1 of month",
  "be", 2561);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M03", day: 31, calendar }).add(months5n),
  2563, 10, "M10", 31, "subtract 5mo with result in the previous year on day 31 of month",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25640716.add(years1months2n),
  2563, 5, "M05", 16, "subtract 1y 2mo",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M02", day: 17, calendar }).add(years1months2n),
  2562, 12, "M12", 17, "subtract 1y 2mo with result in the previous year",
  "be", 2562);

TemporalHelpers.assertPlainDate(
  date25551201.add(months6),
  2556, 6, "M06", 1, "add 6mo",
  "be", 2556);
calculatedStart = date25551201.add(months6).add(months6n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  2555, 12, "M12", 1, "subtract 6mo",
  "be", 2555);

// Weeks

const weeks1 = new Temporal.Duration(0, 0, 1);
const weeks1n = new Temporal.Duration(0, 0, -1);
const weeks6 = new Temporal.Duration(0, 0, 6);
const weeks6n = new Temporal.Duration(0, 0, -6);
const weeks40 = new Temporal.Duration(0, 0, 40);
const weeks40n = new Temporal.Duration(0, 0, -40);
const years1weeks2 = new Temporal.Duration(1, 0, 2);
const years1weeks2n = new Temporal.Duration(-1, 0, -2);
const months2weeks3 = new Temporal.Duration(0, 2, 3);
const months2weeks3n = new Temporal.Duration(0, -2, -3);

const date25550101 = Temporal.PlainDate.from({ year: 2555, monthCode: "M01", day: 1, calendar });
const date25631228 = Temporal.PlainDate.from({ year: 2563, monthCode: "M12", day: 28, calendar });
const date25640127 = Temporal.PlainDate.from({ year: 2564, monthCode: "M01", day: 27, calendar });
const date25640219 = Temporal.PlainDate.from({ year: 2564, monthCode: "M02", day: 19, calendar });
const date25640604 = Temporal.PlainDate.from({ year: 2564, monthCode: "M06", day: 4, calendar });
const date25640627 = Temporal.PlainDate.from({ year: 2564, monthCode: "M06", day: 27, calendar });
const date25640727 = Temporal.PlainDate.from({ year: 2564, monthCode: "M07", day: 27, calendar });
const date25641224 = Temporal.PlainDate.from({ year: 2564, monthCode: "M12", day: 24, calendar });

TemporalHelpers.assertPlainDate(
  date25640219.add(weeks1),
  2564, 2, "M02", 26, "add 1w",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25641224.add(weeks1),
  2564, 12, "M12", 31, "add 1w with result on the last day of the year",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M12", day: 25, calendar }).add(weeks1),
  2565, 1, "M01", 1, "add 1w with result on the first day of the next year",
  "be", 2565);

TemporalHelpers.assertPlainDate(
  date25640127.add(weeks1),
  2564, 2, "M02", 3, "add 1w in a 31-day month with result in the next month",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25640727.add(weeks1),
  2564, 8, "M08", 3, "add 1w in another 31-day month with result in the next month",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25640627.add(weeks1),
  2564, 7, "M07", 4, "add 1w in a 30-day month with result in the next month",
  "be", 2564);

TemporalHelpers.assertPlainDate(
  date25640127.add(weeks6),
  2564, 3, "M03", 10, "add 6w with result in the same year",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25641224.add(weeks6),
  2565, 2, "M02", 4, "add 6w with result in the next year",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  date25640627.add(weeks6),
  2564, 8, "M08", 8, "add 6w crossing months of 30 and 31 days",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25640727.add(weeks6),
  2564, 9, "M09", 7, "add 6w crossing months of 31 and 31 days",
  "be", 2564);

TemporalHelpers.assertPlainDate(
  date25631228.add(years1weeks2),
  2565, 1, "M01", 11, "add 1y 2w with result in the next year",
  "be", 2565);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2562, monthCode: "M10", day: 28, calendar }).add(months2weeks3),
  2563, 1, "M01", 18, "add 2mo 3w with result in the next year",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2562, monthCode: "M10", day: 31, calendar }).add(months2weeks3),
  2563, 1, "M01", 21, "add 2mo 3w with result in the next year",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25640219.add(weeks1n),
  2564, 2, "M02", 12, "subtract 1w",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M01", day: 8, calendar }).add(weeks1n),
  2564, 1, "M01", 1, "subtract 1w with result on the first day of the year",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M01", day: 7, calendar }).add(weeks1n),
  2563, 12, "M12", 31, "subtract 1w with result on the last day of the previous year",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25640604.add(weeks1n),
  2564, 5, "M05", 28, "subtract 1w with result in the previous 31-day month",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M07", day: 3, calendar }).add(weeks1n),
  2564, 6, "M06", 26, "subtract 1w with result in the previous 30-day month",
  "be", 2564);

TemporalHelpers.assertPlainDate(
  date25640604.add(weeks6n),
  2564, 4, "M04", 23, "subtract 6w with result in the same year",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25640127.add(weeks6n),
  2563, 12, "M12", 16, "subtract 6w with result in the previous year",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M09", day: 8, calendar }).add(weeks6n),
  2564, 7, "M07", 28, "subtract 6w crossing months of 30 and 31 days",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M08", day: 8, calendar }).add(weeks6n),
  2564, 6, "M06", 27, "subtract 6w crossing months of 31 and 31 days",
  "be", 2564);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2565, monthCode: "M01", day: 5, calendar }).add(years1weeks2n),
  2563, 12, "M12", 22, "subtract 1y 2w with result in the previous year",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2562, monthCode: "M03", day: 2, calendar }).add(months2weeks3n),
  2561, 12, "M12", 12, "subtract 2mo 3w with result in the previous year",
  "be", 2561);

TemporalHelpers.assertPlainDate(
  date25550101.add(weeks40),
  2555, 10, "M10", 7, "add 40w",
  "be", 2555);
calculatedStart = date25550101.add(weeks40).add(weeks40n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  2555, 1, "M01", 1, "subtract 40w",
  "be", 2555);

// Days

const days10 = new Temporal.Duration(0, 0, 0, 10);
const days10n = new Temporal.Duration(0, 0, 0, -10);
const days280 = new Temporal.Duration(0, 0, 0, 280);
const days280n = new Temporal.Duration(0, 0, 0, -280);
const weeks2days3 = new Temporal.Duration(0, 0, 2, 3);
const weeks2days3n = new Temporal.Duration(0, 0, -2, -3);
const years1months2days4 = new Temporal.Duration(1, 2, 0, 4);
const years1months2days4n = new Temporal.Duration(-1, -2, 0, -4);

TemporalHelpers.assertPlainDate(
  date25640716.add(days10),
  2564, 7, "M07", 26, "add 10 days with result in the same month",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M07", day: 26, calendar }).add(days10),
  2564, 8, "M08", 5, "add 10 days with result in the next month",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M12", day: 26, calendar }).add(days10),
  2565, 1, "M01", 5, "add 10 days with result in the next year",
  "be", 2565);

TemporalHelpers.assertPlainDate(
  date25631228.add(weeks2days3),
  2564, 1, "M01", 14, "add 2w 3d with result in the next year",
  "be", 2564);

TemporalHelpers.assertPlainDate(
  date25640716.add(years1months2days4),
  2565, 9, "M09", 20, "add 1y 2mo 4d",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M02", day: 27, calendar }).add(years1months2days4),
  2565, 5, "M05", 1, "add 1y 2mo 4d with result in a month following a 30-day month",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M07", day: 30, calendar }).add(years1months2days4),
  2565, 10, "M10", 4, "add 1y 2mo 4d with result in a month following a 30-day month",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M01", day: 28, calendar }).add(years1months2days4),
  2565, 4, "M04", 1, "add 1y 2mo 4d with result in a month following a 31-day month",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M06", day: 30, calendar }).add(years1months2days4),
  2565, 9, "M09", 3, "add 1y 2mo 4d with result in a month following a 31-day month",
  "be", 2565);

TemporalHelpers.assertPlainDate(
  date25640716.add(days10n),
  2564, 7, "M07", 6, "subtract 10 days with result in the same month",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M07", day: 6, calendar }).add(days10n),
  2564, 6, "M06", 26, "subtract 10 days with result in the previous month",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M01", day: 4, calendar }).add(days10n),
  2563, 12, "M12", 25, "subtract 10 days with result in the previous year",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M01", day: 15, calendar }).add(weeks2days3n),
  2563, 12, "M12", 29, "subtract 2w 3d with result in the previous year",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25640716.add(years1months2days4n),
  2563, 5, "M05", 12, "subtract 1y 2mo 4d",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M07", day: 4, calendar }).add(years1months2days4n),
  2563, 4, "M04", 30, "subtract 1y 2mo 4d with result in a 30-day month",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25640604.add(years1months2days4n),
  2563, 3, "M03", 31, "subtract 1y 2mo 4d with result in a 31-day month",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25550101.add(days280),
  2555, 10, "M10", 7, "add 280d",
  "be", 2555);
calculatedStart = date25550101.add(days280).add(days280n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  2555, 1, "M01", 1, "subtract 280d",
  "be", 2555);
