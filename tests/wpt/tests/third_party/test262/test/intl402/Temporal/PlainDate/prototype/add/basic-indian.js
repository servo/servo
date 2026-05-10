// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: >
  Check various basic calculations not involving leap years or constraining
  (indian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "indian";

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);
const years3months6days17 = new Temporal.Duration(3, 6, 0, 17);
const years3months6days17n = new Temporal.Duration(-3, -6, 0, -17);

const date19191201 = Temporal.PlainDate.from({ year: 1919, monthCode: "M12", day: 1, calendar });
const date19200716 = Temporal.PlainDate.from({ year: 1920, monthCode: "M07", day: 16, calendar });

TemporalHelpers.assertPlainDate(
  date19200716.add(years1),
  1921, 7, "M07", 16, "add 1y",
  "shaka", 1921);
TemporalHelpers.assertPlainDate(
  date19200716.add(years4),
  1924, 7, "M07", 16, "add 4y",
  "shaka", 1924);

TemporalHelpers.assertPlainDate(
  date19200716.add(years1n),
  1919, 7, "M07", 16, "subtract 1y",
  "shaka", 1919);
TemporalHelpers.assertPlainDate(
  date19200716.add(years4n),
  1916, 7, "M07", 16, "subtract 4y",
  "shaka", 1916);

TemporalHelpers.assertPlainDate(
  date19191201.add(years3months6days17),
  1923, 6, "M06", 18, "Adding 3y6m17d to day 1 of a month",
  "shaka", 1923);
var calculatedStart = date19191201.add(years3months6days17).add(years3months6days17n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  1919, 12, "M12", 1, "subtract 3y6m17d",
  "shaka", 1919);

// Months

const months5 = new Temporal.Duration(0, 5);
const months5n = new Temporal.Duration(0, -5);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);
const months8 = new Temporal.Duration(0, 8);
const months8n = new Temporal.Duration(0, -8);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

const date19220101 = Temporal.PlainDate.from({ year: 1922, monthCode: "M01", day: 1, calendar });
const date19221201 = Temporal.PlainDate.from({ year: 1922, monthCode: "M12", day: 1, calendar });

TemporalHelpers.assertPlainDate(
  date19200716.add(months5),
  1920, 12, "M12", 16, "add 5mo with result in the same year",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M08", day: 16, calendar }).add(months5),
  1921, 1, "M01", 16, "add 5mo with result in the next year",
  "shaka", 1921);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1918, monthCode: "M10", day: 1, calendar }).add(months5),
  1919, 3, "M03", 1, "add 5mo with result in the next year on day 1 of month",
  "shaka", 1919);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M06", day: 31, calendar }).add(months8),
  1921, 2, "M02", 31, "add 8mo with result in the next year on day 31 of month",
  "shaka", 1921);

TemporalHelpers.assertPlainDate(
  date19200716.add(years1months2),
  1921, 9, "M09", 16, "add 1y 2mo",
  "shaka", 1921);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M11", day: 30, calendar }).add(years1months2),
  1922, 1, "M01", 30, "add 1y 2mo with result in the next year",
  "shaka", 1922);

TemporalHelpers.assertPlainDate(
  date19200716.add(months5n),
  1920, 2, "M02", 16, "subtract 5mo with result in the same year",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M01", day: 16, calendar }).add(months5n),
  1919, 8, "M08", 16, "subtract 5mo with result in the previous year",
  "shaka", 1919);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1918, monthCode: "M02", day: 1, calendar }).add(months5n),
  1917, 9, "M09", 1, "subtract 5mo with result in the previous year on day 1 of month",
  "shaka", 1917);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M02", day: 31, calendar }).add(months8n),
  1919, 6, "M06", 31, "subtract 8mo with result in the previous year on day 31 of month",
  "shaka", 1919);

TemporalHelpers.assertPlainDate(
  date19200716.add(years1months2n),
  1919, 5, "M05", 16, "subtract 1y 2mo",
  "shaka", 1919);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M02", day: 17, calendar }).add(years1months2n),
  1918, 12, "M12", 17, "subtract 1y 2mo with result in the previous year",
  "shaka", 1918);

TemporalHelpers.assertPlainDate(
  date19221201.add(months6),
  1923, 6, "M06", 1, "add 6 months, with result in next year",
  "shaka", 1923);
calculatedStart = date19221201.add(months6).add(months6n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  1922, 12, "M12", 1, "subtract 6 months, with result in previous year",
  "shaka", 1922);

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

const date19201228 = Temporal.PlainDate.from({ year: 1920, monthCode: "M12", day: 28, calendar });
const date19200127 = Temporal.PlainDate.from({ year: 1920, monthCode: "M01", day: 27, calendar });
const date19200219 = Temporal.PlainDate.from({ year: 1920, monthCode: "M02", day: 19, calendar });
const date19200527 = Temporal.PlainDate.from({ year: 1920, monthCode: "M05", day: 27, calendar });
const date19200604 = Temporal.PlainDate.from({ year: 1920, monthCode: "M06", day: 4, calendar });
const date19200627 = Temporal.PlainDate.from({ year: 1920, monthCode: "M06", day: 27, calendar });
const date19200704 = Temporal.PlainDate.from({ year: 1920, monthCode: "M07", day: 4, calendar });
const date19200727 = Temporal.PlainDate.from({ year: 1920, monthCode: "M07", day: 27, calendar });
const date19201223 = Temporal.PlainDate.from({ year: 1920, monthCode: "M12", day: 23, calendar });

TemporalHelpers.assertPlainDate(
  date19200219.add(weeks1),
  1920, 2, "M02", 26, "add 1w",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  date19201223.add(weeks1),
  1920, 12, "M12", 30, "add 1w with result on the last day of the year",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M12", day: 24, calendar }).add(weeks1),
  1921, 1, "M01", 1, "add 1w with result on the first day of the next year",
  "shaka", 1921);

TemporalHelpers.assertPlainDate(
  date19200627.add(weeks1),
  1920, 7, "M07", 3, "add 1w in a 31-day month with result in the next month",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  date19200727.add(weeks1),
  1920, 8, "M08", 4, "add 1w in a 30-day month with result in the next month",
  "shaka", 1920);

TemporalHelpers.assertPlainDate(
  date19200127.add(weeks6),
  1920, 3, "M03", 8, "add 6w with result in the same year",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  date19201223.add(weeks6),
  1921, 2, "M02", 5, "add 6w with result in the next year",
  "shaka", 1921);
TemporalHelpers.assertPlainDate(
  date19200127.add(weeks6),
  1920, 3, "M03", 8, "add 6w crossing months of 30 and 31 days",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  date19200527.add(weeks6),
  1920, 7, "M07", 7, "add 6w crossing months of 31 and 31 days",
  "shaka", 1920);

TemporalHelpers.assertPlainDate(
  date19201228.add(years1weeks2),
  1922, 1, "M01", 12, "add 1y 2w with result in the next year",
  "shaka", 1922);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1918, monthCode: "M10", day: 28, calendar }).add(months2weeks3),
  1919, 1, "M01", 19, "add 2mo 3w with result in the next year",
  "shaka", 1919);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1918, monthCode: "M10", day: 31, calendar }).add(months2weeks3),
  1919, 1, "M01", 21, "add 2mo 3w with result in the next year",
  "shaka", 1919);

TemporalHelpers.assertPlainDate(
  date19200219.add(weeks1n),
  1920, 2, "M02", 12, "subtract 1w",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M01", day: 8, calendar }).add(weeks1n),
  1920, 1, "M01", 1, "subtract 1w with result on the first day of the year",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M01", day: 7, calendar }).add(weeks1n),
  1919, 12, "M12", 30, "subtract 1w with result on the last day of the previous year",
  "shaka", 1919);

TemporalHelpers.assertPlainDate(
  date19200704.add(weeks1n),
  1920, 6, "M06", 28, "subtract 1w with result in the previous 31-day month",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M02", day: 3, calendar }).add(weeks1n),
  1920, 1, "M01", 26, "subtract 1w with result in the previous 30-day month",
  "shaka", 1920);

TemporalHelpers.assertPlainDate(
  date19200604.add(weeks6n),
  1920, 4, "M04", 24, "subtract 6w with result in the same year",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  date19200127.add(weeks6n),
  1919, 12, "M12", 15, "subtract 6w with result in the previous year",
  "shaka", 1919);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M07", day: 8, calendar }).add(weeks6n),
  1920, 5, "M05", 28, "subtract 6w crossing months of 30 and 31 days",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M03", day: 8, calendar }).add(weeks6n),
  1920, 1, "M01", 27, "subtract 6w crossing months of 31 and 31 days",
  "shaka", 1920);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1921, monthCode: "M01", day: 5, calendar }).add(years1weeks2n),
  1919, 12, "M12", 21, "subtract 1y 2w with result in the previous year",
  "shaka", 1919);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1918, monthCode: "M03", day: 2, calendar }).add(months2weeks3n),
  1917, 12, "M12", 11, "subtract 2mo 3w with result in the previous year",
  "shaka", 1917);

TemporalHelpers.assertPlainDate(
  date19220101.add(weeks40),
  1922, 10, "M10", 5, "add 40 weeks, ending in same year",
  "shaka", 1922);
calculatedStart = date19220101.add(weeks40).add(weeks40n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  1922, 1, "M01", 1, "subtract 40 weeks, ending in same year",
  "shaka", 1922);

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
  date19200716.add(days10),
  1920, 7, "M07", 26, "add 10 days with result in the same month",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M07", day: 26, calendar }).add(days10),
  1920, 8, "M08", 6, "add 10 days with result in the next month",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M12", day: 26, calendar }).add(days10),
  1921, 1, "M01", 6, "add 10 days with result in the next year",
  "shaka", 1921);

TemporalHelpers.assertPlainDate(
  date19201228.add(weeks2days3),
  1921, 1, "M01", 15, "add 2w 3d with result in the next year",
  "shaka", 1921);

TemporalHelpers.assertPlainDate(
  date19200716.add(years1months2days4),
  1921, 9, "M09", 20, "add 1y 2mo 4d",
  "shaka", 1921);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M11", day: 30, calendar }).add(years1months2days4),
  1922, 2, "M02", 3, "add 1y 2mo 4d with result in a month following a 30-day month",
  "shaka", 1922);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M05", day: 26, calendar }).add(years1months2days4),
  1921, 7, "M07", 30, "add 1y 2mo 4d with result in a month following a 31-day month",
  "shaka", 1921);

TemporalHelpers.assertPlainDate(
  date19200716.add(days10n),
  1920, 7, "M07", 6, "subtract 10 days with result in the same month",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1920, monthCode: "M07", day: 6, calendar }).add(days10n),
  1920, 6, "M06", 27, "subtract 10 days with result in the previous month",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1921, monthCode: "M01", day: 4, calendar }).add(days10n),
  1920, 12, "M12", 24, "subtract 10 days with result in the previous year",
  "shaka", 1920);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1921, monthCode: "M01", day: 15, calendar }).add(weeks2days3n),
  1920, 12, "M12", 28, "subtract 2w 3d with result in the previous year",
  "shaka", 1920);

TemporalHelpers.assertPlainDate(
  date19200716.add(years1months2days4n),
  1919, 5, "M05", 12, "subtract 1y 2mo 4d",
  "shaka", 1919);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1921, monthCode: "M12", day: 4, calendar }).add(years1months2days4n),
  1920, 9, "M09", 30, "subtract 1y 2mo 4d with result in a 30-day month",
  "shaka", 1920);
TemporalHelpers.assertPlainDate(
  date19200604.add(years1months2days4n),
  1919, 3, "M03", 31, "subtract 1y 2mo 4d with result in a 31-day month",
  "shaka", 1919);

TemporalHelpers.assertPlainDate(
  date19220101.add(days280),
  1922, 10, "M10", 5, "add 40 weeks, ending in same year",
  "shaka", 1922);
calculatedStart = date19220101.add(days280).add(days280n);
TemporalHelpers.assertPlainDate(
  calculatedStart,
  1922, 1, "M01", 1, "subtract 40 weeks, ending in same year",
  "shaka", 1922);
