// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: >
  Check various basic calculations not involving leap years or constraining
  (persian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "persian";

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);
const years3months6days17 = new Temporal.Duration(-3, -6, 0, -17);
const years3months6days17n = new Temporal.Duration(3, 6, 0, 17);

const date13751201 = Temporal.PlainDateTime.from({ year: 1375, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar });
const date14000716 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date14010716 = Temporal.PlainDateTime.from({ year: 1401, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(years1),
  1401, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "add 1y",
  "ap", 1401);
TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(years4),
  1404, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "add 4y",
  "ap", 1404);

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(years1n),
  1399, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "subtract 1y",
  "ap", 1399);
TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(years4n),
  1396, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "subtract 4y",
  "ap", 1396);

TemporalHelpers.assertPlainDateTime(
  date13751201.subtract(years3months6days17),
  1379, 6, "M06", 18, 12, 34, 0, 0, 0, 0, "Adding 3y6m17d to day 1 of a month",
  "ap", 1379);
var calculatedStart = date13751201.subtract(years3months6days17).subtract(years3months6days17n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1375, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 3y6m17d",
  "ap", 1375);

// Months

const months5 = new Temporal.Duration(0, -5);
const months5n = new Temporal.Duration(0, 5);
const months6 = new Temporal.Duration(0, -6);
const months6n = new Temporal.Duration(0, 6);
const months8 = new Temporal.Duration(0, -8);
const months8n = new Temporal.Duration(0, 8);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date13781201 = Temporal.PlainDateTime.from({ year: 1378, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar });

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(months5),
  1400, 12, "M12", 16, 12, 34, 0, 0, 0, 0, "add 5mo with result in the same year",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M08", day: 16, hour: 12, minute: 34, calendar }).subtract(months5),
  1401, 1, "M01", 16, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year",
  "ap", 1401);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1398, monthCode: "M10", day: 1, hour: 12, minute: 34, calendar }).subtract(months5),
  1399, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year on day 1 of month",
  "ap", 1399);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M06", day: 31, hour: 12, minute: 34, calendar }).subtract(months8),
  1401, 2, "M02", 31, 12, 34, 0, 0, 0, 0, "add 8mo with result in the next year on day 31 of month",
  "ap", 1401);

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(years1months2),
  1401, 9, "M09", 16, 12, 34, 0, 0, 0, 0, "add 1y 2mo",
  "ap", 1401);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M11", day: 30, hour: 12, minute: 34, calendar }).subtract(years1months2),
  1402, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the next year",
  "ap", 1402);

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(months5n),
  1400, 2, "M02", 16, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the same year",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M01", day: 16, hour: 12, minute: 34, calendar }).subtract(months5n),
  1399, 8, "M08", 16, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the previous year",
  "ap", 1399);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1398, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }).subtract(months5n),
  1397, 9, "M09", 1, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the previous year on day 1 of month",
  "ap", 1397);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M02", day: 31, hour: 12, minute: 34, calendar }).subtract(months8n),
  1399, 6, "M06", 31, 12, 34, 0, 0, 0, 0, "subtract 8mo with result in the previous year on day 31 of month",
  "ap", 1399);

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(years1months2n),
  1399, 5, "M05", 16, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo",
  "ap", 1399);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M02", day: 17, hour: 12, minute: 34, calendar }).subtract(years1months2n),
  1398, 12, "M12", 17, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo with result in the previous year",
  "ap", 1398);

TemporalHelpers.assertPlainDateTime(
  date13781201.subtract(months6),
  1379, 6, "M06", 1, 12, 34, 0, 0, 0, 0, "add 6 months, with result in next year",
  "ap", 1379);
calculatedStart = date13781201.subtract(months6).subtract(months6n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1378, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 6 months, with result in previous year",
  "ap", 1378);

// Weeks

const weeks1 = new Temporal.Duration(0, 0, -1);
const weeks1n = new Temporal.Duration(0, 0, 1);
const weeks6 = new Temporal.Duration(0, 0, -6);
const weeks6n = new Temporal.Duration(0, 0, 6);
const weeks40 = new Temporal.Duration(0, 0, -40);
const weeks40n = new Temporal.Duration(0, 0, 40);
const years1weeks2 = new Temporal.Duration(-1, 0, -2);
const years1weeks2n = new Temporal.Duration(1, 0, 2);
const months2weeks3 = new Temporal.Duration(0, -2, -3);
const months2weeks3n = new Temporal.Duration(0, 2, 3);

const date13780101 = Temporal.PlainDateTime.from({ year: 1378, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar });
const date13991228 = Temporal.PlainDateTime.from({ year: 1399, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar });
const date14000219 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M02", day: 19, hour: 12, minute: 34, calendar });
const date14000527 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M05", day: 27, hour: 12, minute: 34, calendar });
const date14000604 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M06", day: 4, hour: 12, minute: 34, calendar });
const date14000627 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M06", day: 27, hour: 12, minute: 34, calendar });
const date14000704 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M07", day: 4, hour: 12, minute: 34, calendar });
const date14000727 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M07", day: 27, hour: 12, minute: 34, calendar });
const date14001122 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M11", day: 22, hour: 12, minute: 34, calendar });
const date14001127 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M11", day: 27, hour: 12, minute: 34, calendar });
const date14001222 = Temporal.PlainDateTime.from({ year: 1400, monthCode: "M12", day: 22, hour: 12, minute: 34, calendar });
const date14010127 = Temporal.PlainDateTime.from({ year: 1401, monthCode: "M01", day: 27, hour: 12, minute: 34, calendar });
const date14010604 = Temporal.PlainDateTime.from({ year: 1401, monthCode: "M06", day: 4, hour: 12, minute: 34, calendar });

TemporalHelpers.assertPlainDateTime(
  date14000219.subtract(weeks1),
  1400, 2, "M02", 26, 12, 34, 0, 0, 0, 0, "add 1w",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  date14001222.subtract(weeks1),
  1400, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "add 1w with result on the last day of the year",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M12", day: 23, hour: 12, minute: 34, calendar }).subtract(weeks1),
  1401, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "add 1w with result on the first day of the next year",
  "ap", 1401);

TemporalHelpers.assertPlainDateTime(
  date14000627.subtract(weeks1),
  1400, 7, "M07", 3, 12, 34, 0, 0, 0, 0, "add 1w in a 31-day month with result in the next month",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  date14000727.subtract(weeks1),
  1400, 8, "M08", 4, 12, 34, 0, 0, 0, 0, "add 1w in a 30-day month with result in the next month",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  date14010127.subtract(weeks6),
  1401, 3, "M03", 7, 12, 34, 0, 0, 0, 0, "add 6w with result in the same year",
  "ap", 1401);
TemporalHelpers.assertPlainDateTime(
  date14001222.subtract(weeks6),
  1401, 2, "M02", 4, 12, 34, 0, 0, 0, 0, "add 6w with result in the next year",
  "ap", 1401);
TemporalHelpers.assertPlainDateTime(
  date14001127.subtract(weeks6),
  1401, 1, "M01", 10, 12, 34, 0, 0, 0, 0, "add 6w crossing months of 30 and 31 days",
  "ap", 1401);
TemporalHelpers.assertPlainDateTime(
  date14000627.subtract(weeks6),
  1400, 8, "M08", 8, 12, 34, 0, 0, 0, 0, "add 6w crossing months of 31 and 31 days",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  date13991228.subtract(years1weeks2),
  1401, 1, "M01", 13, 12, 34, 0, 0, 0, 0, "add 1y 2w with result in the next year",
  "ap", 1401);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1398, monthCode: "M10", day: 28, hour: 12, minute: 34, calendar }).subtract(months2weeks3),
  1399, 1, "M01", 20, 12, 34, 0, 0, 0, 0, "add 2mo 3w with result in the next year",
  "ap", 1399);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1398, monthCode: "M10", day: 31, hour: 12, minute: 34, calendar }).subtract(months2weeks3),
  1399, 1, "M01", 21, 12, 34, 0, 0, 0, 0, "add 2mo 3w with result in the next year",
  "ap", 1399);

TemporalHelpers.assertPlainDateTime(
  date14000219.subtract(weeks1n),
  1400, 2, "M02", 12, 12, 34, 0, 0, 0, 0, "subtract 1w",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M01", day: 8, hour: 12, minute: 34, calendar }).subtract(weeks1n),
  1400, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 1w with result on the first day of the year",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1401, monthCode: "M01", day: 7, hour: 12, minute: 34, calendar }).subtract(weeks1n),
  1400, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "subtract 1w with result on the last day of the previous year",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  date14000704.subtract(weeks1n),
  1400, 6, "M06", 28, 12, 34, 0, 0, 0, 0, "subtract 1w with result in the previous 31-day month",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M12", day: 3, hour: 12, minute: 34, calendar }).subtract(weeks1n),
  1400, 11, "M11", 26, 12, 34, 0, 0, 0, 0, "subtract 1w with result in the previous 30-day month",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  date14000604.subtract(weeks6n),
  1400, 4, "M04", 24, 12, 34, 0, 0, 0, 0, "subtract 6w with result in the same year",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  date14010127.subtract(weeks6n),
  1400, 12, "M12", 14, 12, 34, 0, 0, 0, 0, "subtract 6w with result in the previous year",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M07", day: 8, hour: 12, minute: 34, calendar }).subtract(weeks6n),
  1400, 5, "M05", 28, 12, 34, 0, 0, 0, 0, "subtract 6w crossing months of 30 and 31 days",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M03", day: 8, hour: 12, minute: 34, calendar }).subtract(weeks6n),
  1400, 1, "M01", 28, 12, 34, 0, 0, 0, 0, "subtract 6w crossing months of 31 and 31 days",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1402, monthCode: "M01", day: 5, hour: 12, minute: 34, calendar }).subtract(years1weeks2n),
  1400, 12, "M12", 20, 12, 34, 0, 0, 0, 0, "subtract 1y 2w with result in the previous year",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1398, monthCode: "M03", day: 2, hour: 12, minute: 34, calendar }).subtract(months2weeks3n),
  1397, 12, "M12", 10, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w with result in the previous year",
  "ap", 1397);

TemporalHelpers.assertPlainDateTime(
  date13780101.subtract(weeks40),
  1378, 10, "M10", 5, 12, 34, 0, 0, 0, 0, "add 40 weeks, ending in same year",
  "ap", 1378);
calculatedStart = date13780101.subtract(weeks40).subtract(weeks40n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1378, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 40 weeks, ending in same year",
  "ap", 1378);

// Days

const days10 = new Temporal.Duration(0, 0, 0, -10);
const days10n = new Temporal.Duration(0, 0, 0, 10);
const days280 = new Temporal.Duration(0, 0, 0, -280);
const days280n = new Temporal.Duration(0, 0, 0, 280);
const weeks2days3 = new Temporal.Duration(0, 0, -2, -3);
const weeks2days3n = new Temporal.Duration(0, 0, 2, 3);
const years1months2days4 = new Temporal.Duration(-1, -2, 0, -4);
const years1months2days4n = new Temporal.Duration(1, 2, 0, 4);

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(days10),
  1400, 7, "M07", 26, 12, 34, 0, 0, 0, 0, "add 10 days with result in the same month",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M07", day: 26, hour: 12, minute: 34, calendar }).subtract(days10),
  1400, 8, "M08", 6, 12, 34, 0, 0, 0, 0, "add 10 days with result in the next month",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M12", day: 26, hour: 12, minute: 34, calendar }).subtract(days10),
  1401, 1, "M01", 7, 12, 34, 0, 0, 0, 0, "add 10 days with result in the next year",
  "ap", 1401);

TemporalHelpers.assertPlainDateTime(
  date13991228.subtract(weeks2days3),
  1400, 1, "M01", 15, 12, 34, 0, 0, 0, 0, "add 2w 3d with result in the next year",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(years1months2days4),
  1401, 9, "M09", 20, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d",
  "ap", 1401);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1399, monthCode: "M10", day: 20, hour: 12, minute: 34, calendar }).subtract(years1months2days4),
  1400, 12, "M12", 24, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 30-day month",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M05", day: 26, hour: 12, minute: 34, calendar }).subtract(years1months2days4),
  1401, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 31-day month",
  "ap", 1401);

TemporalHelpers.assertPlainDateTime(
  date14000716.subtract(days10n),
  1400, 7, "M07", 6, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the same month",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1400, monthCode: "M07", day: 6, hour: 12, minute: 34, calendar }).subtract(days10n),
  1400, 6, "M06", 27, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the previous month",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1401, monthCode: "M01", day: 4, hour: 12, minute: 34, calendar }).subtract(days10n),
  1400, 12, "M12", 23, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the previous year",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1401, monthCode: "M01", day: 15, hour: 12, minute: 34, calendar }).subtract(weeks2days3n),
  1400, 12, "M12", 27, 12, 34, 0, 0, 0, 0, "subtract 2w 3d with result in the previous year",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  date14010716.subtract(years1months2days4n),
  1400, 5, "M05", 12, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1401, monthCode: "M12", day: 4, hour: 12, minute: 34, calendar }).subtract(years1months2days4n),
  1400, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in a 30-day month",
  "ap", 1400);
TemporalHelpers.assertPlainDateTime(
  date14010604.subtract(years1months2days4n),
  1400, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in a 31-day month",
  "ap", 1400);

TemporalHelpers.assertPlainDateTime(
  date13780101.subtract(days280),
  1378, 10, "M10", 5, 12, 34, 0, 0, 0, 0, "add 280 days, ending in same year",
  "ap", 1378);
calculatedStart = date13780101.subtract(days280).subtract(days280n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart,
  1378, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 280 days, ending in same year",
  "ap", 1378);
