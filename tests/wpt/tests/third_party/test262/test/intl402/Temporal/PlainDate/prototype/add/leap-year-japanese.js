// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Check various basic calculations involving leap years (japanese calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

const date20200229 = Temporal.PlainDate.from({ year: 2020, monthCode: "M02", day: 29, calendar }, options);

TemporalHelpers.assertPlainDate(
  date20200229.add(years1),
  2021, 2, "M02", 28, "add 1y to leap day and constrain",
  "reiwa", 3);
assert.throws(RangeError, function () {
  date20200229.add(years1, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDate(
  date20200229.add(years4, options),
  2024, 2, "M02", 29, "add 4y to leap day",
  "reiwa", 6);

TemporalHelpers.assertPlainDate(
  date20200229.add(years1n),
  2019, 2, "M02", 28, "subtract 1y from leap day and constrain",
  "heisei", 31);
assert.throws(RangeError, function () {
  date20200229.add(years1n, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDate(
  date20200229.add(years4n, options),
  2016, 2, "M02", 29, "subtract 4y from leap day",
  "heisei", 28);

// Months

const months1 = new Temporal.Duration(0, 1);
const months1n = new Temporal.Duration(0, -1);
const months5 = new Temporal.Duration(0, 5);
const months11n = new Temporal.Duration(0, -11);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

const date20200131 = Temporal.PlainDate.from({ year: 2020, monthCode: "M01", day: 31, calendar }, options);
const date20200331 = Temporal.PlainDate.from({ year: 2020, monthCode: "M03", day: 31, calendar }, options);

TemporalHelpers.assertPlainDate(
  date20200131.add(months1),
  2020, 2, "M02", 29, "add 1mo to Jan 31 constrains to Feb 29 in leap year",
  "reiwa", 2);
assert.throws(RangeError, function () {
  date20200131.add(months1, options);
}, "add 1mo to Jan 31 rejects");

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M09", day: 30, calendar }, options).add(months5),
  2022, 2, "M02", 28, "add 5mo with result in the next year constrained to Feb 28",
  "reiwa", 4);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2019, monthCode: "M09", day: 30, calendar }, options).add(months5),
  2020, 2, "M02", 29, "add 5mo with result in the next year constrained to Feb 29",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M12", day: 31, calendar }, options).add(years1months2),
  2023, 2, "M02", 28, "add 1y 2mo with result in the next year constrained to Feb 28",
  "reiwa", 5);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2022, monthCode: "M12", day: 31, calendar }, options).add(years1months2),
  2024, 2, "M02", 29, "add 1y 2mo with result in the next year constrained to Feb 29",
  "reiwa", 6);

TemporalHelpers.assertPlainDate(
  date20200331.add(months1n),
  2020, 2, "M02", 29, "subtract 1mo from Mar 31 constrains to Feb 29 in leap year",
  "reiwa", 2);
assert.throws(RangeError, function () {
  date20200331.add(months1n, options);
}, "subtract 1mo from Mar 31 rejects");

TemporalHelpers.assertPlainDate(
  date20200131.add(months11n),
  2019, 2, "M02", 28, "subtract 11mo with result in the previous year constrained to Feb 28",
  "heisei", 31);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M01", day: 31, calendar }, options).add(months11n),
  2020, 2, "M02", 29, "add 11mo with result in the next year constrained to Feb 29",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2022, monthCode: "M04", day: 30, calendar }, options).add(years1months2n),
  2021, 2, "M02", 28, "add 1y 2mo with result in the previous year constrained to Feb 28",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M04", day: 30, calendar }, options).add(years1months2n),
  2020, 2, "M02", 29, "add 1y 2mo with result in the previous year constrained to Feb 29",
  "reiwa", 2);

// Weeks

const weeks1 = new Temporal.Duration(0, 0, 1);
const weeks1n = new Temporal.Duration(0, 0, -1);
const weeks6 = new Temporal.Duration(0, 0, 6);
const weeks6n = new Temporal.Duration(0, 0, -6);
const years1weeks2 = new Temporal.Duration(1, 0, 2);
const years1weeks2n = new Temporal.Duration(-1, 0, -2);
const months2weeks3 = new Temporal.Duration(0, 2, 3);
const months2weeks3n = new Temporal.Duration(0, -2, -3);
const months11weeks3n = new Temporal.Duration(0, -11, -3);

const date20191228 = Temporal.PlainDate.from({ year: 2019, monthCode: "M12", day: 28, calendar }, options);
const date20200219 = Temporal.PlainDate.from({ year: 2020, monthCode: "M02", day: 19, calendar }, options);
const date20200228 = Temporal.PlainDate.from({ year: 2020, monthCode: "M02", day: 28, calendar }, options);
const date20200301 = Temporal.PlainDate.from({ year: 2020, monthCode: "M03", day: 1, calendar }, options);
const date20200303 = Temporal.PlainDate.from({ year: 2020, monthCode: "M03", day: 3, calendar }, options);
const date20201228 = Temporal.PlainDate.from({ year: 2020, monthCode: "M12", day: 28, calendar }, options);
const date20210219 = Temporal.PlainDate.from({ year: 2021, monthCode: "M02", day: 19, calendar }, options);
const date20210228 = Temporal.PlainDate.from({ year: 2021, monthCode: "M02", day: 28, calendar }, options);
const date20210301 = Temporal.PlainDate.from({ year: 2021, monthCode: "M03", day: 1, calendar }, options);
const date20210303 = Temporal.PlainDate.from({ year: 2021, monthCode: "M03", day: 3, calendar }, options);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M02", day: 27, calendar }, options).add(weeks1),
  2021, 3, "M03", 6, "add 1w in Feb with result in March",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2020, monthCode: "M02", day: 27, calendar }, options).add(weeks1),
  2020, 3, "M03", 5, "add 1w in Feb with result in March in a leap year",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  date20210219.add(weeks6),
  2021, 4, "M04", 2, "add 6w in Feb with result in the next month",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  date20200219.add(weeks6),
  2020, 4, "M04", 1, "add 6w in Feb with result in the next month in a leap year",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2020, monthCode: "M01", day: 27, calendar }, options).add(weeks6),
  2020, 3, "M03", 9, "add 6w with result in the same year, crossing leap day",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  date20200228.add(years1weeks2),
  2021, 3, "M03", 14, "add 1y 2w to Feb 28 with result in March starting in leap year",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  date20210228.add(years1weeks2),
  2022, 3, "M03", 14, "add 1y 2w to Feb 28 with result in March starting in common year",
  "reiwa", 4);
TemporalHelpers.assertPlainDate(
  date20200229.add(years1weeks2),
  2021, 3, "M03", 14, "add 1y 2w to Feb 29 with result in March",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2019, monthCode: "M02", day: 28, calendar }, options).add(years1weeks2),
  2020, 3, "M03", 13, "add 1y 2w to Feb 28 with result in March ending in leap year",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  date20200229.add(months2weeks3),
  2020, 5, "M05", 20, "add 2mo 3w to leap day",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  date20200228.add(months2weeks3),
  2020, 5, "M05", 19, "add 2mo 3w to Feb 28 of a leap year",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  date20210228.add(months2weeks3),
  2021, 5, "M05", 19, "add 2mo 3w to Feb 28 of a common year",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  date20201228.add(months2weeks3),
  2021, 3, "M03", 21, "add 2mo 3w from end of year crossing common-year Feb",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  date20191228.add(months2weeks3),
  2020, 3, "M03", 20, "add 2mo 3w from end of year crossing leap-year Feb",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  date20210303.add(weeks1n),
  2021, 2, "M02", 24, "subtract 1w in March with result in Feb",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  date20200303.add(weeks1n),
  2020, 2, "M02", 25, "subtract 1w in March with result in leap-year Feb",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M04", day: 2, calendar }, options).add(weeks6n),
  2021, 2, "M02", 19, "subtract 6w with result in Feb",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2020, monthCode: "M04", day: 2, calendar }, options).add(weeks6n),
  2020, 2, "M02", 20, "subtract 6w with result in leap-year Feb",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2020, monthCode: "M03", day: 9, calendar }, options).add(weeks6n),
  2020, 1, "M01", 27, "subtract 6w with result in the same year, crossing leap day",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  date20200301.add(years1weeks2n),
  2019, 2, "M02", 15, "subtract 1y 2w from Mar 1 starting in leap year",
  "heisei", 31);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2023, monthCode: "M03", day: 1, calendar }, options).add(years1weeks2n),
  2022, 2, "M02", 15, "subtract 1y 2w from Mar 1 starting in common year",
  "reiwa", 4);
TemporalHelpers.assertPlainDate(
  date20200229.add(years1weeks2n),
  2019, 2, "M02", 14, "subtract 1y 2w from Feb 29",
  "heisei", 31);
TemporalHelpers.assertPlainDate(
  date20210301.add(years1weeks2n),
  2020, 2, "M02", 16, "subtract 1y 2w from Mar 1 ending in leap year",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  date20200229.add(months2weeks3n),
  2019, 12, "M12", 8, "subtract 2mo 3w from leap day",
  "reiwa", 1);
TemporalHelpers.assertPlainDate(
  date20200301.add(months2weeks3n),
  2019, 12, "M12", 11, "subtract 2mo 3w from Mar 1 of a leap year",
  "reiwa", 1);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2019, monthCode: "M03", day: 1, calendar }, options).add(months2weeks3n),
  2018, 12, "M12", 11, "subtract 2mo 3w from Mar 1 of a common year",
  "heisei", 30);
TemporalHelpers.assertPlainDate(
  date20191228.add(months11weeks3n),
  2019, 1, "M01", 7, "add 2mo 3w from end of year crossing common-year Feb",
  "heisei", 31);
TemporalHelpers.assertPlainDate(
  date20201228.add(months11weeks3n),
  2020, 1, "M01", 7, "add 2mo 3w from end of year crossing leap-year Feb",
  "reiwa", 2);

// Days

const days10 = new Temporal.Duration(0, 0, 0, 10);
const days10n = new Temporal.Duration(0, 0, 0, -10);
const weeks2days3 = new Temporal.Duration(0, 0, 2, 3);
const weeks2days3n = new Temporal.Duration(0, 0, -2, -3);
const years1months2days4 = new Temporal.Duration(1, 2, 0, 4);
const years1months2days4n = new Temporal.Duration(-1, -2, 0, -4);

const date20210226 = Temporal.PlainDate.from({ year: 2021, monthCode: "M02", day: 26, calendar }, options);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2020, monthCode: "M02", day: 26, calendar }, options).add(days10),
  2020, 3, "M03", 7, "add 10d crossing leap day",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  date20210226.add(days10),
  2021, 3, "M03", 8, "add 10d crossing end of common-year Feb",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  date20200219.add(days10),
  2020, 2, "M02", 29, "add 10d with result on leap day",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  date20210219.add(days10),
  2021, 3, "M03", 1, "add 10d with result on common-year March 1",
  "reiwa", 3);

TemporalHelpers.assertPlainDate(
  date20200229.add(weeks2days3),
  2020, 3, "M03", 17, "add 2w 3d to leap day",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  date20210228.add(weeks2days3),
  2021, 3, "M03", 17, "add 2w 3d to end of common-year Feb",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  date20200228.add(weeks2days3),
  2020, 3, "M03", 16, "add 2w 3d to day before leap day",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  date20210226.add(years1months2days4),
  2022, 4, "M04", 30, "add 1y 2mo 4d with result in common-year April",
  "reiwa", 4);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2023, monthCode: "M02", day: 26, calendar }, options).add(years1months2days4),
  2024, 4, "M04", 30, "add 1y 2mo 4d with result in leap-year April",
  "reiwa", 6);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M12", day: 30, calendar }, options).add(years1months2days4),
  2023, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into common-year March",
  "reiwa", 5);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2022, monthCode: "M12", day: 30, calendar }, options).add(years1months2days4),
  2024, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into leap-year March",
  "reiwa", 6);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2022, monthCode: "M12", day: 29, calendar }, options).add(years1months2days4),
  2024, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into leap-year March",
  "reiwa", 6);

TemporalHelpers.assertPlainDate(
  date20200303.add(days10n),
  2020, 2, "M02", 22, "subtract 10d crossing leap day",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  date20210303.add(days10n),
  2021, 2, "M02", 21, "subtract 10d crossing end of common-year Feb",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2020, monthCode: "M03", day: 10, calendar }, options).add(days10n),
  2020, 2, "M02", 29, "subtract 10d with result on leap day",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M03", day: 10, calendar }, options).add(days10n),
  2021, 2, "M02", 28, "subtract 10d with result on common-year Feb 28",
  "reiwa", 3);

TemporalHelpers.assertPlainDate(
  date20200229.add(weeks2days3n),
  2020, 2, "M02", 12, "subtract 2w 3d from leap day",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  date20210301.add(weeks2days3n),
  2021, 2, "M02", 12, "subtract 2w 3d from common-year Mar 1",
  "reiwa", 3);
TemporalHelpers.assertPlainDate(
  date20200301.add(weeks2days3n),
  2020, 2, "M02", 13, "subtract 2w 3d from leap-year Mar 1",
  "reiwa", 2);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2023, monthCode: "M03", day: 24, calendar }, options).add(years1months2days4n),
  2022, 1, "M01", 20, "subtract 1y 2mo 4d with result in common-year January",
  "reiwa", 4);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M03", day: 24, calendar }, options).add(years1months2days4n),
  2020, 1, "M01", 20, "subtract 1y 2mo 4d with result in leap-year January",
  "reiwa", 2);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2023, monthCode: "M05", day: 1, calendar }, options).add(years1months2days4n),
  2022, 2, "M02", 25, "add 1y 2mo 4d with result rolling over into common-year February",
  "reiwa", 4);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2021, monthCode: "M05", day: 1, calendar }, options).add(years1months2days4n),
  2020, 2, "M02", 26, "add 1y 2mo 4d with result rolling over into leap-year February",
  "reiwa", 2);
