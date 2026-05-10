// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: >
  Check various basic calculations not involving leap years or constraining
  (japanese calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "japanese";

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);
const years3months6days17 = new Temporal.Duration(3, 6, 0, 17);
const years3months6days17n = new Temporal.Duration(-3, -6, 0, -17);

const date19971201 = Temporal.PlainDateTime.from({ year: 1997, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar });
const date20210716 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });

TemporalHelpers.assertPlainDateTime(
  date20210716.add(years1),
  2022, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "add 1y",
  "reiwa", 4);
TemporalHelpers.assertPlainDateTime(
  date20210716.add(years4),
  2025, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "add 4y",
  "reiwa", 7);

TemporalHelpers.assertPlainDateTime(
  date20210716.add(years1n),
  2020, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "subtract 1y",
  "reiwa", 2);
TemporalHelpers.assertPlainDateTime(
  date20210716.add(years4n),
  2017, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "subtract 4y",
  "heisei", 29);

TemporalHelpers.assertPlainDateTime(
  date19971201.add(years3months6days17),
  2001, 6, "M06", 18, 12, 34, 0, 0, 0, 0, "Adding 3y6m17d to day 1 of a month",
  "heisei", 13);
var calculatedStart = date19971201.add(years3months6days17).add(years3months6days17n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1997, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 3y6m17d",
  "heisei", 9);

// Months

const months5 = new Temporal.Duration(0, 5);
const months5n = new Temporal.Duration(0, -5);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

const date20001201 = Temporal.PlainDateTime.from({ year: 2000, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar });

TemporalHelpers.assertPlainDateTime(
  date20210716.add(months5),
  2021, 12, "M12", 16, 12, 34, 0, 0, 0, 0, "add 5mo with result in the same year",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M08", day: 16, hour: 12, minute: 34, calendar }).add(months5),
  2022, 1, "M01", 16, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year",
  "reiwa", 4);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2019, monthCode: "M10", day: 1, hour: 12, minute: 34, calendar }).add(months5),
  2020, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year on day 1 of month",
  "reiwa", 2);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M10", day: 31, hour: 12, minute: 34, calendar }).add(months5),
  2022, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year on day 31 of month",
  "reiwa", 4);

TemporalHelpers.assertPlainDateTime(
  date20210716.add(years1months2),
  2022, 9, "M09", 16, 12, 34, 0, 0, 0, 0, "add 1y 2mo",
  "reiwa", 4);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M11", day: 30, hour: 12, minute: 34, calendar }).add(years1months2),
  2023, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the next year",
  "reiwa", 5);

TemporalHelpers.assertPlainDateTime(
  date20210716.add(months5n),
  2021, 2, "M02", 16, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the same year",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 16, hour: 12, minute: 34, calendar }).add(months5n),
  2020, 8, "M08", 16, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the previous year",
  "reiwa", 2);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2019, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }).add(months5n),
  2018, 9, "M09", 1, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the previous year on day 1 of month",
  "heisei", 30);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M03", day: 31, hour: 12, minute: 34, calendar }).add(months5n),
  2020, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the previous year on day 31 of month",
  "reiwa", 2);

TemporalHelpers.assertPlainDateTime(
  date20210716.add(years1months2n),
  2020, 5, "M05", 16, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo",
  "reiwa", 2);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 17, hour: 12, minute: 34, calendar }).add(years1months2n),
  2019, 12, "M12", 17, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo with result in the previous year",
  "reiwa", 1);

TemporalHelpers.assertPlainDateTime(
  date20001201.add(months6),
  2001, 6, "M06", 1, 12, 34, 0, 0, 0, 0, "add 6 months, with result in next year",
  "heisei", 13);
calculatedStart = date20001201.add(months6).add(months6n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  2000, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 6 months, with result in previous year",
  "heisei", 12);

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

const date20000101 = Temporal.PlainDateTime.from({ year: 2000, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar });
const date20201228 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar });
const date20210127 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 27, hour: 12, minute: 34, calendar });
const date20210219 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 19, hour: 12, minute: 34, calendar });
const date20210604 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M06", day: 4, hour: 12, minute: 34, calendar });
const date20210627 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M06", day: 27, hour: 12, minute: 34, calendar });
const date20210727 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 27, hour: 12, minute: 34, calendar });
const date20211224 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M12", day: 24, hour: 12, minute: 34, calendar });

TemporalHelpers.assertPlainDateTime(
  date20210219.add(weeks1),
  2021, 2, "M02", 26, 12, 34, 0, 0, 0, 0, "add 1w",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  date20211224.add(weeks1),
  2021, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "add 1w with result on the last day of the year",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M12", day: 25, hour: 12, minute: 34, calendar }).add(weeks1),
  2022, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "add 1w with result on the first day of the next year",
  "reiwa", 4);

TemporalHelpers.assertPlainDateTime(
  date20210127.add(weeks1),
  2021, 2, "M02", 3, 12, 34, 0, 0, 0, 0, "add 1w in a 31-day month with result in the next month",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  date20210727.add(weeks1),
  2021, 8, "M08", 3, 12, 34, 0, 0, 0, 0, "add 1w in another 31-day month with result in the next month",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  date20210627.add(weeks1),
  2021, 7, "M07", 4, 12, 34, 0, 0, 0, 0, "add 1w in a 30-day month with result in the next month",
  "reiwa", 3);

TemporalHelpers.assertPlainDateTime(
  date20210127.add(weeks6),
  2021, 3, "M03", 10, 12, 34, 0, 0, 0, 0, "add 6w with result in the same year",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  date20211224.add(weeks6),
  2022, 2, "M02", 4, 12, 34, 0, 0, 0, 0, "add 6w with result in the next year",
  "reiwa", 4);
TemporalHelpers.assertPlainDateTime(
  date20210627.add(weeks6),
  2021, 8, "M08", 8, 12, 34, 0, 0, 0, 0, "add 6w crossing months of 30 and 31 days",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  date20210727.add(weeks6),
  2021, 9, "M09", 7, 12, 34, 0, 0, 0, 0, "add 6w crossing months of 31 and 31 days",
  "reiwa", 3);

TemporalHelpers.assertPlainDateTime(
  date20201228.add(years1weeks2),
  2022, 1, "M01", 11, 12, 34, 0, 0, 0, 0, "add 1y 2w with result in the next year",
  "reiwa", 4);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2019, monthCode: "M10", day: 28, hour: 12, minute: 34, calendar }).add(months2weeks3),
  2020, 1, "M01", 18, 12, 34, 0, 0, 0, 0, "add 2mo 3w with result in the next year",
  "reiwa", 2);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2019, monthCode: "M10", day: 31, hour: 12, minute: 34, calendar }).add(months2weeks3),
  2020, 1, "M01", 21, 12, 34, 0, 0, 0, 0, "add 2mo 3w with result in the next year",
  "reiwa", 2);

TemporalHelpers.assertPlainDateTime(
  date20210219.add(weeks1n),
  2021, 2, "M02", 12, 12, 34, 0, 0, 0, 0, "subtract 1w",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 8, hour: 12, minute: 34, calendar }).add(weeks1n),
  2021, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 1w with result on the first day of the year",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 7, hour: 12, minute: 34, calendar }).add(weeks1n),
  2020, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "subtract 1w with result on the last day of the previous year",
  "reiwa", 2);

TemporalHelpers.assertPlainDateTime(
  date20210604.add(weeks1n),
  2021, 5, "M05", 28, 12, 34, 0, 0, 0, 0, "subtract 1w with result in the previous 31-day month",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 3, hour: 12, minute: 34, calendar }).add(weeks1n),
  2021, 6, "M06", 26, 12, 34, 0, 0, 0, 0, "subtract 1w with result in the previous 30-day month",
  "reiwa", 3);

TemporalHelpers.assertPlainDateTime(
  date20210604.add(weeks6n),
  2021, 4, "M04", 23, 12, 34, 0, 0, 0, 0, "subtract 6w with result in the same year",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  date20210127.add(weeks6n),
  2020, 12, "M12", 16, 12, 34, 0, 0, 0, 0, "subtract 6w with result in the previous year",
  "reiwa", 2);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M09", day: 8, hour: 12, minute: 34, calendar }).add(weeks6n),
  2021, 7, "M07", 28, 12, 34, 0, 0, 0, 0, "subtract 6w crossing months of 30 and 31 days",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M08", day: 8, hour: 12, minute: 34, calendar }).add(weeks6n),
  2021, 6, "M06", 27, 12, 34, 0, 0, 0, 0, "subtract 6w crossing months of 31 and 31 days",
  "reiwa", 3);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2022, monthCode: "M01", day: 5, hour: 12, minute: 34, calendar }).add(years1weeks2n),
  2020, 12, "M12", 22, 12, 34, 0, 0, 0, 0, "subtract 1y 2w with result in the previous year",
  "reiwa", 2);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2019, monthCode: "M03", day: 2, hour: 12, minute: 34, calendar }).add(months2weeks3n),
  2018, 12, "M12", 12, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w with result in the previous year",
  "heisei", 30);

TemporalHelpers.assertPlainDateTime(
  date20000101.add(weeks40),
  2000, 10, "M10", 7, 12, 34, 0, 0, 0, 0, "add 40 weeks, ending in same year",
  "heisei", 12);
calculatedStart = date20000101.add(weeks40).add(weeks40n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  2000, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 40 weeks, ending in same year",
  "heisei", 12);

// Days

const days10 = new Temporal.Duration(0, 0, 0, 10);
const days10n = new Temporal.Duration(0, 0, 0, -10);
const days280 = new Temporal.Duration(0, 0, 0, 280);
const days280n = new Temporal.Duration(0, 0, 0, -280);
const weeks2days3 = new Temporal.Duration(0, 0, 2, 3);
const weeks2days3n = new Temporal.Duration(0, 0, -2, -3);
const years1months2days4 = new Temporal.Duration(1, 2, 0, 4);
const years1months2days4n = new Temporal.Duration(-1, -2, 0, -4);

TemporalHelpers.assertPlainDateTime(
  date20210716.add(days10),
  2021, 7, "M07", 26, 12, 34, 0, 0, 0, 0, "add 10 days with result in the same month",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 26, hour: 12, minute: 34, calendar }).add(days10),
  2021, 8, "M08", 5, 12, 34, 0, 0, 0, 0, "add 10 days with result in the next month",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M12", day: 26, hour: 12, minute: 34, calendar }).add(days10),
  2022, 1, "M01", 5, 12, 34, 0, 0, 0, 0, "add 10 days with result in the next year",
  "reiwa", 4);

TemporalHelpers.assertPlainDateTime(
  date20201228.add(weeks2days3),
  2021, 1, "M01", 14, 12, 34, 0, 0, 0, 0, "add 2w 3d with result in the next year",
  "reiwa", 3);

TemporalHelpers.assertPlainDateTime(
  date20210716.add(years1months2days4),
  2022, 9, "M09", 20, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d",
  "reiwa", 4);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 27, hour: 12, minute: 34, calendar }).add(years1months2days4),
  2022, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 30-day month",
  "reiwa", 4);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 30, hour: 12, minute: 34, calendar }).add(years1months2days4),
  2022, 10, "M10", 4, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 30-day month",
  "reiwa", 4);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 28, hour: 12, minute: 34, calendar }).add(years1months2days4),
  2022, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 31-day month",
  "reiwa", 4);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M06", day: 30, hour: 12, minute: 34, calendar }).add(years1months2days4),
  2022, 9, "M09", 3, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 31-day month",
  "reiwa", 4);

TemporalHelpers.assertPlainDateTime(
  date20210716.add(days10n),
  2021, 7, "M07", 6, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the same month",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 6, hour: 12, minute: 34, calendar }).add(days10n),
  2021, 6, "M06", 26, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the previous month",
  "reiwa", 3);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 4, hour: 12, minute: 34, calendar }).add(days10n),
  2020, 12, "M12", 25, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the previous year",
  "reiwa", 2);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 15, hour: 12, minute: 34, calendar }).add(weeks2days3n),
  2020, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "subtract 2w 3d with result in the previous year",
  "reiwa", 2);

TemporalHelpers.assertPlainDateTime(
  date20210716.add(years1months2days4n),
  2020, 5, "M05", 12, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d",
  "reiwa", 2);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 4, hour: 12, minute: 34, calendar }).add(years1months2days4n),
  2020, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in a 30-day month",
  "reiwa", 2);
TemporalHelpers.assertPlainDateTime(
  date20210604.add(years1months2days4n),
  2020, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in a 31-day month",
  "reiwa", 2);

TemporalHelpers.assertPlainDateTime(
  date20000101.add(days280),
  2000, 10, "M10", 7, 12, 34, 0, 0, 0, 0, "add 280 days, ending in same year",
  "heisei", 12);
calculatedStart = date20000101.add(days280).add(days280n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  2000, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 280 days, ending in same year",
  "heisei", 12);
