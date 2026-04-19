// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Check various basic calculations involving leap years (gregory calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "gregory";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

const date20200229 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M02", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(years1).toPlainDateTime(),
  2021, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "add 1y to leap day and constrain",
  "ce", 2021);
assert.throws(RangeError, function () {
  date20200229.subtract(years1, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(years4, options).toPlainDateTime(),
  2024, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "add 4y to leap day",
  "ce", 2024);

TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(years1n).toPlainDateTime(),
  2019, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "subtract 1y from leap day and constrain",
  "ce", 2019);
assert.throws(RangeError, function () {
  date20200229.subtract(years1n, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(years4n, options).toPlainDateTime(),
  2016, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "subtract 4y from leap day",
  "ce", 2016);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months5 = new Temporal.Duration(0, -5);
const months11n = new Temporal.Duration(0, 11);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date20200131 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200331 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M03", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date20200131.subtract(months1).toPlainDateTime(),
  2020, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "add 1mo to Jan 31 constrains to Feb 29 in leap year",
  "ce", 2020);
assert.throws(RangeError, function () {
  date20200131.subtract(months1, options);
}, "add 1mo to Jan 31 rejects");

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M09", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(months5).toPlainDateTime(),
  2022, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year constrained to Feb 28",
  "ce", 2022);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M09", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(months5).toPlainDateTime(),
  2020, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "add 5mo with result in the next year constrained to Feb 29",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2).toPlainDateTime(),
  2023, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the next year constrained to Feb 28",
  "ce", 2023);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2022, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2).toPlainDateTime(),
  2024, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the next year constrained to Feb 29",
  "ce", 2024);

TemporalHelpers.assertPlainDateTime(
  date20200331.subtract(months1n).toPlainDateTime(),
  2020, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "subtract 1mo from Mar 31 constrains to Feb 29 in leap year",
  "ce", 2020);
assert.throws(RangeError, function () {
  date20200331.subtract(months1n, options);
}, "subtract 1mo from Mar 31 rejects");

TemporalHelpers.assertPlainDateTime(
  date20200131.subtract(months11n).toPlainDateTime(),
  2019, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "subtract 11mo with result in the previous year constrained to Feb 28",
  "ce", 2019);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(months11n).toPlainDateTime(),
  2020, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "add 11mo with result in the next year constrained to Feb 29",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2022, monthCode: "M04", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2n).toPlainDateTime(),
  2021, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the previous year constrained to Feb 28",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M04", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2n).toPlainDateTime(),
  2020, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the previous year constrained to Feb 29",
  "ce", 2020);

// Weeks

const weeks1 = new Temporal.Duration(0, 0, -1);
const weeks1n = new Temporal.Duration(0, 0, 1);
const weeks6 = new Temporal.Duration(0, 0, -6);
const weeks6n = new Temporal.Duration(0, 0, 6);
const years1weeks2 = new Temporal.Duration(-1, 0, -2);
const years1weeks2n = new Temporal.Duration(1, 0, 2);
const months2weeks3 = new Temporal.Duration(0, -2, -3);
const months2weeks3n = new Temporal.Duration(0, 2, 3);
const months11weeks3n = new Temporal.Duration(0, 11, 3);

const date20191228 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M12", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200219 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M02", day: 19, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200228 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200301 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20200303 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M03", day: 3, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20201228 = Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M12", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210219 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M02", day: 19, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210228 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210301 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date20210303 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M03", day: 3, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M02", day: 27, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(weeks1).toPlainDateTime(),
  2021, 3, "M03", 6, 12, 34, 0, 0, 0, 0, "add 1w in Feb with result in March",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M02", day: 27, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(weeks1).toPlainDateTime(),
  2020, 3, "M03", 5, 12, 34, 0, 0, 0, 0, "add 1w in Feb with result in March in a leap year",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  date20210219.subtract(weeks6).toPlainDateTime(),
  2021, 4, "M04", 2, 12, 34, 0, 0, 0, 0, "add 6w in Feb with result in the next month",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  date20200219.subtract(weeks6).toPlainDateTime(),
  2020, 4, "M04", 1, 12, 34, 0, 0, 0, 0, "add 6w in Feb with result in the next month in a leap year",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M01", day: 27, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(weeks6).toPlainDateTime(),
  2020, 3, "M03", 9, 12, 34, 0, 0, 0, 0, "add 6w with result in the same year, crossing leap day",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  date20200228.subtract(years1weeks2).toPlainDateTime(),
  2021, 3, "M03", 14, 12, 34, 0, 0, 0, 0, "add 1y 2w to Feb 28 with result in March starting in leap year",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  date20210228.subtract(years1weeks2).toPlainDateTime(),
  2022, 3, "M03", 14, 12, 34, 0, 0, 0, 0, "add 1y 2w to Feb 28 with result in March starting in common year",
  "ce", 2022);
TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(years1weeks2).toPlainDateTime(),
  2021, 3, "M03", 14, 12, 34, 0, 0, 0, 0, "add 1y 2w to Feb 29 with result in March",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1weeks2).toPlainDateTime(),
  2020, 3, "M03", 13, 12, 34, 0, 0, 0, 0, "add 1y 2w to Feb 28 with result in March ending in leap year",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(months2weeks3).toPlainDateTime(),
  2020, 5, "M05", 20, 12, 34, 0, 0, 0, 0, "add 2mo 3w to leap day",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  date20200228.subtract(months2weeks3).toPlainDateTime(),
  2020, 5, "M05", 19, 12, 34, 0, 0, 0, 0, "add 2mo 3w to Feb 28 of a leap year",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  date20210228.subtract(months2weeks3).toPlainDateTime(),
  2021, 5, "M05", 19, 12, 34, 0, 0, 0, 0, "add 2mo 3w to Feb 28 of a common year",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  date20201228.subtract(months2weeks3).toPlainDateTime(),
  2021, 3, "M03", 21, 12, 34, 0, 0, 0, 0, "add 2mo 3w from end of year crossing common-year Feb",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  date20191228.subtract(months2weeks3).toPlainDateTime(),
  2020, 3, "M03", 20, 12, 34, 0, 0, 0, 0, "add 2mo 3w from end of year crossing leap-year Feb",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  date20210303.subtract(weeks1n).toPlainDateTime(),
  2021, 2, "M02", 24, 12, 34, 0, 0, 0, 0, "subtract 1w in March with result in Feb",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  date20200303.subtract(weeks1n).toPlainDateTime(),
  2020, 2, "M02", 25, 12, 34, 0, 0, 0, 0, "subtract 1w in March with result in leap-year Feb",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M04", day: 2, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(weeks6n).toPlainDateTime(),
  2021, 2, "M02", 19, 12, 34, 0, 0, 0, 0, "subtract 6w with result in Feb",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M04", day: 2, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(weeks6n).toPlainDateTime(),
  2020, 2, "M02", 20, 12, 34, 0, 0, 0, 0, "subtract 6w with result in leap-year Feb",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M03", day: 9, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(weeks6n).toPlainDateTime(),
  2020, 1, "M01", 27, 12, 34, 0, 0, 0, 0, "subtract 6w with result in the same year, crossing leap day",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  date20200301.subtract(years1weeks2n).toPlainDateTime(),
  2019, 2, "M02", 15, 12, 34, 0, 0, 0, 0, "subtract 1y 2w from Mar 1 starting in leap year",
  "ce", 2019);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2023, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1weeks2n).toPlainDateTime(),
  2022, 2, "M02", 15, 12, 34, 0, 0, 0, 0, "subtract 1y 2w from Mar 1 starting in common year",
  "ce", 2022);
TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(years1weeks2n).toPlainDateTime(),
  2019, 2, "M02", 14, 12, 34, 0, 0, 0, 0, "subtract 1y 2w from Feb 29",
  "ce", 2019);
TemporalHelpers.assertPlainDateTime(
  date20210301.subtract(years1weeks2n).toPlainDateTime(),
  2020, 2, "M02", 16, 12, 34, 0, 0, 0, 0, "subtract 1y 2w from Mar 1 ending in leap year",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(months2weeks3n).toPlainDateTime(),
  2019, 12, "M12", 8, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w from leap day",
  "ce", 2019);
TemporalHelpers.assertPlainDateTime(
  date20200301.subtract(months2weeks3n).toPlainDateTime(),
  2019, 12, "M12", 11, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w from Mar 1 of a leap year",
  "ce", 2019);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M03", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(months2weeks3n).toPlainDateTime(),
  2018, 12, "M12", 11, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w from Mar 1 of a common year",
  "ce", 2018);
TemporalHelpers.assertPlainDateTime(
  date20191228.subtract(months11weeks3n).toPlainDateTime(),
  2019, 1, "M01", 7, 12, 34, 0, 0, 0, 0, "add 2mo 3w from end of year crossing common-year Feb",
  "ce", 2019);
TemporalHelpers.assertPlainDateTime(
  date20201228.subtract(months11weeks3n).toPlainDateTime(),
  2020, 1, "M01", 7, 12, 34, 0, 0, 0, 0, "add 2mo 3w from end of year crossing leap-year Feb",
  "ce", 2020);

// Days

const days10 = new Temporal.Duration(0, 0, 0, -10);
const days10n = new Temporal.Duration(0, 0, 0, 10);
const weeks2days3 = new Temporal.Duration(0, 0, -2, -3);
const weeks2days3n = new Temporal.Duration(0, 0, 2, 3);
const years1months2days4 = new Temporal.Duration(-1, -2, 0, -4);
const years1months2days4n = new Temporal.Duration(1, 2, 0, 4);

const date20210226 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M02", day: 26, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M02", day: 26, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(days10).toPlainDateTime(),
  2020, 3, "M03", 7, 12, 34, 0, 0, 0, 0, "add 10d crossing leap day",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  date20210226.subtract(days10).toPlainDateTime(),
  2021, 3, "M03", 8, 12, 34, 0, 0, 0, 0, "add 10d crossing end of common-year Feb",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  date20200219.subtract(days10).toPlainDateTime(),
  2020, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "add 10d with result on leap day",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  date20210219.subtract(days10).toPlainDateTime(),
  2021, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "add 10d with result on common-year March 1",
  "ce", 2021);

TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(weeks2days3).toPlainDateTime(),
  2020, 3, "M03", 17, 12, 34, 0, 0, 0, 0, "add 2w 3d to leap day",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  date20210228.subtract(weeks2days3).toPlainDateTime(),
  2021, 3, "M03", 17, 12, 34, 0, 0, 0, 0, "add 2w 3d to end of common-year Feb",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  date20200228.subtract(weeks2days3).toPlainDateTime(),
  2020, 3, "M03", 16, 12, 34, 0, 0, 0, 0, "add 2w 3d to day before leap day",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  date20210226.subtract(years1months2days4).toPlainDateTime(),
  2022, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in common-year April",
  "ce", 2022);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2023, monthCode: "M02", day: 26, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2days4).toPlainDateTime(),
  2024, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in leap-year April",
  "ce", 2024);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2days4).toPlainDateTime(),
  2023, 3, "M03", 4, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into common-year March",
  "ce", 2023);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2022, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2days4).toPlainDateTime(),
  2024, 3, "M03", 4, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into leap-year March",
  "ce", 2024);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2022, monthCode: "M12", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2days4).toPlainDateTime(),
  2024, 3, "M03", 4, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into leap-year March",
  "ce", 2024);

TemporalHelpers.assertPlainDateTime(
  date20200303.subtract(days10n).toPlainDateTime(),
  2020, 2, "M02", 22, 12, 34, 0, 0, 0, 0, "subtract 10d crossing leap day",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  date20210303.subtract(days10n).toPlainDateTime(),
  2021, 2, "M02", 21, 12, 34, 0, 0, 0, 0, "subtract 10d crossing end of common-year Feb",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2020, monthCode: "M03", day: 10, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(days10n).toPlainDateTime(),
  2020, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "subtract 10d with result on leap day",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M03", day: 10, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(days10n).toPlainDateTime(),
  2021, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "subtract 10d with result on common-year Feb 28",
  "ce", 2021);

TemporalHelpers.assertPlainDateTime(
  date20200229.subtract(weeks2days3n).toPlainDateTime(),
  2020, 2, "M02", 12, 12, 34, 0, 0, 0, 0, "subtract 2w 3d from leap day",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  date20210301.subtract(weeks2days3n).toPlainDateTime(),
  2021, 2, "M02", 12, 12, 34, 0, 0, 0, 0, "subtract 2w 3d from common-year Mar 1",
  "ce", 2021);
TemporalHelpers.assertPlainDateTime(
  date20200301.subtract(weeks2days3n).toPlainDateTime(),
  2020, 2, "M02", 13, 12, 34, 0, 0, 0, 0, "subtract 2w 3d from leap-year Mar 1",
  "ce", 2020);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2023, monthCode: "M03", day: 24, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2days4n).toPlainDateTime(),
  2022, 1, "M01", 20, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in common-year January",
  "ce", 2022);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M03", day: 24, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2days4n).toPlainDateTime(),
  2020, 1, "M01", 20, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in leap-year January",
  "ce", 2020);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2023, monthCode: "M05", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2days4n).toPlainDateTime(),
  2022, 2, "M02", 25, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into common-year February",
  "ce", 2022);
TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M05", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options).subtract(years1months2days4n).toPlainDateTime(),
  2020, 2, "M02", 26, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into leap-year February",
  "ce", 2020);
