// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: >
  Check various basic calculations not involving leap years or constraining
  (roc calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "roc";

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);
const years3months6days17 = new Temporal.Duration(3, 6, 0, 17);
const years3months6days17n = new Temporal.Duration(-3, -6, 0, -17);

const date861201 = Temporal.ZonedDateTime.from({ year: 86, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110716 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });

TemporalHelpers.assertPlainDateTime(
  date1110716.add(years1).toPlainDateTime(),
  112, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "add 1y",
  "roc", 112);
TemporalHelpers.assertPlainDateTime(
  date1110716.add(years4).toPlainDateTime(),
  115, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "add 4y",
  "roc", 115);

TemporalHelpers.assertPlainDateTime(
  date1110716.add(years1n).toPlainDateTime(),
  110, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "subtract 1y",
  "roc", 110);
TemporalHelpers.assertPlainDateTime(
  date1110716.add(years4n).toPlainDateTime(),
  107, 7, "M07", 16, 12, 34, 0, 0, 0, 0, "subtract 4y",
  "roc", 107);

TemporalHelpers.assertPlainDateTime(
  date861201.add(years3months6days17).toPlainDateTime(),
  90, 6, "M06", 18, 12, 34, 0, 0, 0, 0, "Adding 3y6m17d to day 1 of a month",
  "roc", 90);
var calculatedStart = date861201.add(years3months6days17).add(years3months6days17n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  86, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 3y6m17d",
  "roc", 86);


// Months

const months5 = new Temporal.Duration(0, 5);
const months5n = new Temporal.Duration(0, -5);
const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

const date901201 = Temporal.ZonedDateTime.from({ year: 90, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });

TemporalHelpers.assertPlainDateTime(
  date1110716.add(months5).toPlainDateTime(),
  111, 12, "M12", 16, 12, 34, 0, 0, 0, 0, "add 5mo with result in the same year",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M08", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months5).toPlainDateTime(),
  112, 1, "M01", 16, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year",
  "roc", 112);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 109, monthCode: "M10", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months5).toPlainDateTime(),
  110, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year on day 1 of month",
  "roc", 110);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M10", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months5).toPlainDateTime(),
  112, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year on day 31 of month",
  "roc", 112);

TemporalHelpers.assertPlainDateTime(
  date1110716.add(years1months2).toPlainDateTime(),
  112, 9, "M09", 16, 12, 34, 0, 0, 0, 0, "add 1y 2mo",
  "roc", 112);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M11", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(years1months2).toPlainDateTime(),
  113, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the next year",
  "roc", 113);

TemporalHelpers.assertPlainDateTime(
  date1110716.add(months5n).toPlainDateTime(),
  111, 2, "M02", 16, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the same year",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M01", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months5n).toPlainDateTime(),
  110, 8, "M08", 16, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the previous year",
  "roc", 110);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 109, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months5n).toPlainDateTime(),
  108, 9, "M09", 1, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the previous year on day 1 of month",
  "roc", 108);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M03", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months5n).toPlainDateTime(),
  110, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "subtract 5mo with result in the previous year on day 31 of month",
  "roc", 110);

TemporalHelpers.assertPlainDateTime(
  date1110716.add(years1months2n).toPlainDateTime(),
  110, 5, "M05", 16, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo",
  "roc", 110);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M02", day: 17, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(years1months2n).toPlainDateTime(),
  109, 12, "M12", 17, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo with result in the previous year",
  "roc", 109);

TemporalHelpers.assertPlainDateTime(
  date901201.add(months6).toPlainDateTime(),
  91, 6, "M06", 1, 12, 34, 0, 0, 0, 0, "add 6 months, with result in next year",
  "roc", 91);
calculatedStart = date901201.add(months6).add(months6n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  90, 12, "M12", 1, 12, 34, 0, 0, 0, 0, "subtract 6 months, with result in previous year",
  "roc", 90);

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

const date890101 = Temporal.ZonedDateTime.from({ year: 89, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1101228 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M12", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110127 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M01", day: 27, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110219 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M02", day: 19, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110604 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M06", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110627 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M06", day: 27, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110727 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 27, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1111224 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M12", day: 24, hour: 12, minute: 34, timeZone: "UTC", calendar });

TemporalHelpers.assertPlainDateTime(
  date1110219.add(weeks1).toPlainDateTime(),
  111, 2, "M02", 26, 12, 34, 0, 0, 0, 0, "add 1w",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  date1111224.add(weeks1).toPlainDateTime(),
  111, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "add 1w with result on the last day of the year",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M12", day: 25, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(weeks1).toPlainDateTime(),
  112, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "add 1w with result on the first day of the next year",
  "roc", 112);

TemporalHelpers.assertPlainDateTime(
  date1110127.add(weeks1).toPlainDateTime(),
  111, 2, "M02", 3, 12, 34, 0, 0, 0, 0, "add 1w in a 31-day month with result in the next month",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  date1110727.add(weeks1).toPlainDateTime(),
  111, 8, "M08", 3, 12, 34, 0, 0, 0, 0, "add 1w in another 31-day month with result in the next month",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  date1110627.add(weeks1).toPlainDateTime(),
  111, 7, "M07", 4, 12, 34, 0, 0, 0, 0, "add 1w in a 30-day month with result in the next month",
  "roc", 111);

TemporalHelpers.assertPlainDateTime(
  date1110127.add(weeks6).toPlainDateTime(),
  111, 3, "M03", 10, 12, 34, 0, 0, 0, 0, "add 6w with result in the same year",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  date1111224.add(weeks6).toPlainDateTime(),
  112, 2, "M02", 4, 12, 34, 0, 0, 0, 0, "add 6w with result in the next year",
  "roc", 112);
TemporalHelpers.assertPlainDateTime(
  date1110627.add(weeks6).toPlainDateTime(),
  111, 8, "M08", 8, 12, 34, 0, 0, 0, 0, "add 6w crossing months of 30 and 31 days",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  date1110727.add(weeks6).toPlainDateTime(),
  111, 9, "M09", 7, 12, 34, 0, 0, 0, 0, "add 6w crossing months of 31 and 31 days",
  "roc", 111);

TemporalHelpers.assertPlainDateTime(
  date1101228.add(years1weeks2).toPlainDateTime(),
  112, 1, "M01", 11, 12, 34, 0, 0, 0, 0, "add 1y 2w with result in the next year",
  "roc", 112);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 109, monthCode: "M10", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months2weeks3).toPlainDateTime(),
  110, 1, "M01", 18, 12, 34, 0, 0, 0, 0, "add 2mo 3w with result in the next year",
  "roc", 110);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 109, monthCode: "M10", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months2weeks3).toPlainDateTime(),
  110, 1, "M01", 21, 12, 34, 0, 0, 0, 0, "add 2mo 3w with result in the next year",
  "roc", 110);

TemporalHelpers.assertPlainDateTime(
  date1110219.add(weeks1n).toPlainDateTime(),
  111, 2, "M02", 12, 12, 34, 0, 0, 0, 0, "subtract 1w",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M01", day: 8, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(weeks1n).toPlainDateTime(),
  111, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 1w with result on the first day of the year",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M01", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(weeks1n).toPlainDateTime(),
  110, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "subtract 1w with result on the last day of the previous year",
  "roc", 110);

TemporalHelpers.assertPlainDateTime(
  date1110604.add(weeks1n).toPlainDateTime(),
  111, 5, "M05", 28, 12, 34, 0, 0, 0, 0, "subtract 1w with result in the previous 31-day month",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 3, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(weeks1n).toPlainDateTime(),
  111, 6, "M06", 26, 12, 34, 0, 0, 0, 0, "subtract 1w with result in the previous 30-day month",
  "roc", 111);

TemporalHelpers.assertPlainDateTime(
  date1110604.add(weeks6n).toPlainDateTime(),
  111, 4, "M04", 23, 12, 34, 0, 0, 0, 0, "subtract 6w with result in the same year",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  date1110127.add(weeks6n).toPlainDateTime(),
  110, 12, "M12", 16, 12, 34, 0, 0, 0, 0, "subtract 6w with result in the previous year",
  "roc", 110);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M09", day: 8, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(weeks6n).toPlainDateTime(),
  111, 7, "M07", 28, 12, 34, 0, 0, 0, 0, "subtract 6w crossing months of 30 and 31 days",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M08", day: 8, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(weeks6n).toPlainDateTime(),
  111, 6, "M06", 27, 12, 34, 0, 0, 0, 0, "subtract 6w crossing months of 31 and 31 days",
  "roc", 111);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 112, monthCode: "M01", day: 5, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(years1weeks2n).toPlainDateTime(),
  110, 12, "M12", 22, 12, 34, 0, 0, 0, 0, "subtract 1y 2w with result in the previous year",
  "roc", 110);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 109, monthCode: "M03", day: 2, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(months2weeks3n).toPlainDateTime(),
  108, 12, "M12", 12, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w with result in the previous year",
  "roc", 108);

TemporalHelpers.assertPlainDateTime(
  date890101.add(weeks40).toPlainDateTime(),
  89, 10, "M10", 7, 12, 34, 0, 0, 0, 0, "add 40 weeks, ending in same year",
  "roc", 89);
calculatedStart = date890101.add(weeks40).add(weeks40n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  89, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 40 weeks, ending in same year",
  "roc", 89);

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
  date1110716.add(days10).toPlainDateTime(),
  111, 7, "M07", 26, 12, 34, 0, 0, 0, 0, "add 10 days with result in the same month",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 26, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(days10).toPlainDateTime(),
  111, 8, "M08", 5, 12, 34, 0, 0, 0, 0, "add 10 days with result in the next month",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M12", day: 26, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(days10).toPlainDateTime(),
  112, 1, "M01", 5, 12, 34, 0, 0, 0, 0, "add 10 days with result in the next year",
  "roc", 112);

TemporalHelpers.assertPlainDateTime(
  date1101228.add(weeks2days3).toPlainDateTime(),
  111, 1, "M01", 14, 12, 34, 0, 0, 0, 0, "add 2w 3d with result in the next year",
  "roc", 111);

TemporalHelpers.assertPlainDateTime(
  date1110716.add(years1months2days4).toPlainDateTime(),
  112, 9, "M09", 20, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d",
  "roc", 112);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M02", day: 27, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(years1months2days4).toPlainDateTime(),
  112, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 30-day month",
  "roc", 112);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(years1months2days4).toPlainDateTime(),
  112, 10, "M10", 4, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 30-day month",
  "roc", 112);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M01", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(years1months2days4).toPlainDateTime(),
  112, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 31-day month",
  "roc", 112);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M06", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(years1months2days4).toPlainDateTime(),
  112, 9, "M09", 3, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in a month following a 31-day month",
  "roc", 112);

TemporalHelpers.assertPlainDateTime(
  date1110716.add(days10n).toPlainDateTime(),
  111, 7, "M07", 6, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the same month",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 6, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(days10n).toPlainDateTime(),
  111, 6, "M06", 26, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the previous month",
  "roc", 111);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M01", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(days10n).toPlainDateTime(),
  110, 12, "M12", 25, 12, 34, 0, 0, 0, 0, "subtract 10 days with result in the previous year",
  "roc", 110);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M01", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(weeks2days3n).toPlainDateTime(),
  110, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "subtract 2w 3d with result in the previous year",
  "roc", 110);

TemporalHelpers.assertPlainDateTime(
  date1110716.add(years1months2days4n).toPlainDateTime(),
  110, 5, "M05", 12, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d",
  "roc", 110);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar }).add(years1months2days4n).toPlainDateTime(),
  110, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in a 30-day month",
  "roc", 110);
TemporalHelpers.assertPlainDateTime(
  date1110604.add(years1months2days4n).toPlainDateTime(),
  110, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in a 31-day month",
  "roc", 110);

TemporalHelpers.assertPlainDateTime(
  date890101.add(days280).toPlainDateTime(),
  89, 10, "M10", 7, 12, 34, 0, 0, 0, 0, "add 280 days, ending in same year",
  "roc", 89);
calculatedStart = date890101.add(days280).add(days280n);
TemporalHelpers.assertPlainDateTime(
  calculatedStart.toPlainDateTime(),
  89, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "subtract 280 days, ending in same year",
  "roc", 89);
