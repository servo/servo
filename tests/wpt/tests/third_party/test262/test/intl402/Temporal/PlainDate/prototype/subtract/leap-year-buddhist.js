// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Check various basic calculations involving leap years (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

const date25630229 = Temporal.PlainDate.from({ year: 2563, monthCode: "M02", day: 29, calendar }, options);

TemporalHelpers.assertPlainDate(
  date25630229.subtract(years1),
  2564, 2, "M02", 28, "add 1y to leap day and constrain",
  "be", 2564);
assert.throws(RangeError, function () {
  date25630229.subtract(years1, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDate(
  date25630229.subtract(years4, options),
  2567, 2, "M02", 29, "add 4y to leap day",
  "be", 2567);

TemporalHelpers.assertPlainDate(
  date25630229.subtract(years1n),
  2562, 2, "M02", 28, "subtract 1y from leap day and constrain",
  "be", 2562);
assert.throws(RangeError, function () {
  date25630229.subtract(years1n, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDate(
  date25630229.subtract(years4n, options),
  2559, 2, "M02", 29, "subtract 4y from leap day",
  "be", 2559);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months5 = new Temporal.Duration(0, -5);
const months11n = new Temporal.Duration(0, 11);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date25630131 = Temporal.PlainDate.from({ year: 2563, monthCode: "M01", day: 31, calendar }, options);
const date25630331 = Temporal.PlainDate.from({ year: 2563, monthCode: "M03", day: 31, calendar }, options);

TemporalHelpers.assertPlainDate(
  date25630131.subtract(months1),
  2563, 2, "M02", 29, "add 1mo to Jan 31 constrains to Feb 29 in leap year",
  "be", 2563);
assert.throws(RangeError, function () {
  date25630131.subtract(months1, options);
}, "add 1mo to Jan 31 rejects");

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M09", day: 30, calendar }, options).subtract(months5),
  2565, 2, "M02", 28, "add 5mo with result in the next year constrained to Feb 28",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2562, monthCode: "M09", day: 30, calendar }, options).subtract(months5),
  2563, 2, "M02", 29, "add 5mo with result in the next year constrained to Feb 29",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M12", day: 31, calendar }, options).subtract(years1months2),
  2566, 2, "M02", 28, "add 1y 2mo with result in the next year constrained to Feb 28",
  "be", 2566);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2565, monthCode: "M12", day: 31, calendar }, options).subtract(years1months2),
  2567, 2, "M02", 29, "add 1y 2mo with result in the next year constrained to Feb 29",
  "be", 2567);

TemporalHelpers.assertPlainDate(
  date25630331.subtract(months1n),
  2563, 2, "M02", 29, "subtract 1mo from Mar 31 constrains to Feb 29 in leap year",
  "be", 2563);
assert.throws(RangeError, function () {
  date25630331.subtract(months1n, options);
}, "subtract 1mo from Mar 31 rejects");

TemporalHelpers.assertPlainDate(
  date25630131.subtract(months11n),
  2562, 2, "M02", 28, "subtract 11mo with result in the previous year constrained to Feb 28",
  "be", 2562);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M01", day: 31, calendar }, options).subtract(months11n),
  2563, 2, "M02", 29, "add 11mo with result in the next year constrained to Feb 29",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2565, monthCode: "M04", day: 30, calendar }, options).subtract(years1months2n),
  2564, 2, "M02", 28, "add 1y 2mo with result in the previous year constrained to Feb 28",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M04", day: 30, calendar }, options).subtract(years1months2n),
  2563, 2, "M02", 29, "add 1y 2mo with result in the previous year constrained to Feb 29",
  "be", 2563);

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

const date25621228 = Temporal.PlainDate.from({ year: 2562, monthCode: "M12", day: 28, calendar }, options);
const date25630219 = Temporal.PlainDate.from({ year: 2563, monthCode: "M02", day: 19, calendar }, options);
const date25630228 = Temporal.PlainDate.from({ year: 2563, monthCode: "M02", day: 28, calendar }, options);
const date25630301 = Temporal.PlainDate.from({ year: 2563, monthCode: "M03", day: 1, calendar }, options);
const date25630303 = Temporal.PlainDate.from({ year: 2563, monthCode: "M03", day: 3, calendar }, options);
const date25631228 = Temporal.PlainDate.from({ year: 2563, monthCode: "M12", day: 28, calendar }, options);
const date25640219 = Temporal.PlainDate.from({ year: 2564, monthCode: "M02", day: 19, calendar }, options);
const date25640228 = Temporal.PlainDate.from({ year: 2564, monthCode: "M02", day: 28, calendar }, options);
const date25640301 = Temporal.PlainDate.from({ year: 2564, monthCode: "M03", day: 1, calendar }, options);
const date25640303 = Temporal.PlainDate.from({ year: 2564, monthCode: "M03", day: 3, calendar }, options);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M02", day: 27, calendar }, options).subtract(weeks1),
  2564, 3, "M03", 6, "add 1w in Feb with result in March",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2563, monthCode: "M02", day: 27, calendar }, options).subtract(weeks1),
  2563, 3, "M03", 5, "add 1w in Feb with result in March in a leap year",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25640219.subtract(weeks6),
  2564, 4, "M04", 2, "add 6w in Feb with result in the next month",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25630219.subtract(weeks6),
  2563, 4, "M04", 1, "add 6w in Feb with result in the next month in a leap year",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2563, monthCode: "M01", day: 27, calendar }, options).subtract(weeks6),
  2563, 3, "M03", 9, "add 6w with result in the same year, crossing leap day",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25630228.subtract(years1weeks2),
  2564, 3, "M03", 14, "add 1y 2w to Feb 28 with result in March starting in leap year",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25640228.subtract(years1weeks2),
  2565, 3, "M03", 14, "add 1y 2w to Feb 28 with result in March starting in common year",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  date25630229.subtract(years1weeks2),
  2564, 3, "M03", 14, "add 1y 2w to Feb 29 with result in March",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2562, monthCode: "M02", day: 28, calendar }, options).subtract(years1weeks2),
  2563, 3, "M03", 13, "add 1y 2w to Feb 28 with result in March ending in leap year",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25630229.subtract(months2weeks3),
  2563, 5, "M05", 20, "add 2mo 3w to leap day",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25630228.subtract(months2weeks3),
  2563, 5, "M05", 19, "add 2mo 3w to Feb 28 of a leap year",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25640228.subtract(months2weeks3),
  2564, 5, "M05", 19, "add 2mo 3w to Feb 28 of a common year",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25631228.subtract(months2weeks3),
  2564, 3, "M03", 21, "add 2mo 3w from end of year crossing common-year Feb",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25621228.subtract(months2weeks3),
  2563, 3, "M03", 20, "add 2mo 3w from end of year crossing leap-year Feb",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25640303.subtract(weeks1n),
  2564, 2, "M02", 24, "subtract 1w in March with result in Feb",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25630303.subtract(weeks1n),
  2563, 2, "M02", 25, "subtract 1w in March with result in leap-year Feb",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M04", day: 2, calendar }, options).subtract(weeks6n),
  2564, 2, "M02", 19, "subtract 6w with result in Feb",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2563, monthCode: "M04", day: 2, calendar }, options).subtract(weeks6n),
  2563, 2, "M02", 20, "subtract 6w with result in leap-year Feb",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2563, monthCode: "M03", day: 9, calendar }, options).subtract(weeks6n),
  2563, 1, "M01", 27, "subtract 6w with result in the same year, crossing leap day",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25630301.subtract(years1weeks2n),
  2562, 2, "M02", 15, "subtract 1y 2w from Mar 1 starting in leap year",
  "be", 2562);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2566, monthCode: "M03", day: 1, calendar }, options).subtract(years1weeks2n),
  2565, 2, "M02", 15, "subtract 1y 2w from Mar 1 starting in common year",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  date25630229.subtract(years1weeks2n),
  2562, 2, "M02", 14, "subtract 1y 2w from Feb 29",
  "be", 2562);
TemporalHelpers.assertPlainDate(
  date25640301.subtract(years1weeks2n),
  2563, 2, "M02", 16, "subtract 1y 2w from Mar 1 ending in leap year",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25630229.subtract(months2weeks3n),
  2562, 12, "M12", 8, "subtract 2mo 3w from leap day",
  "be", 2562);
TemporalHelpers.assertPlainDate(
  date25630301.subtract(months2weeks3n),
  2562, 12, "M12", 11, "subtract 2mo 3w from Mar 1 of a leap year",
  "be", 2562);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2562, monthCode: "M03", day: 1, calendar }, options).subtract(months2weeks3n),
  2561, 12, "M12", 11, "subtract 2mo 3w from Mar 1 of a common year",
  "be", 2561);
TemporalHelpers.assertPlainDate(
  date25621228.subtract(months11weeks3n),
  2562, 1, "M01", 7, "add 2mo 3w from end of year crossing common-year Feb",
  "be", 2562);
TemporalHelpers.assertPlainDate(
  date25631228.subtract(months11weeks3n),
  2563, 1, "M01", 7, "add 2mo 3w from end of year crossing leap-year Feb",
  "be", 2563);

// Days

const days10 = new Temporal.Duration(0, 0, 0, -10);
const days10n = new Temporal.Duration(0, 0, 0, 10);
const weeks2days3 = new Temporal.Duration(0, 0, -2, -3);
const weeks2days3n = new Temporal.Duration(0, 0, 2, 3);
const years1months2days4 = new Temporal.Duration(-1, -2, 0, -4);
const years1months2days4n = new Temporal.Duration(1, 2, 0, 4);

const date25640226 = Temporal.PlainDate.from({ year: 2564, monthCode: "M02", day: 26, calendar }, options);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2563, monthCode: "M02", day: 26, calendar }, options).subtract(days10),
  2563, 3, "M03", 7, "add 10d crossing leap day",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25640226.subtract(days10),
  2564, 3, "M03", 8, "add 10d crossing end of common-year Feb",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25630219.subtract(days10),
  2563, 2, "M02", 29, "add 10d with result on leap day",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25640219.subtract(days10),
  2564, 3, "M03", 1, "add 10d with result on common-year March 1",
  "be", 2564);

TemporalHelpers.assertPlainDate(
  date25630229.subtract(weeks2days3),
  2563, 3, "M03", 17, "add 2w 3d to leap day",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25640228.subtract(weeks2days3),
  2564, 3, "M03", 17, "add 2w 3d to end of common-year Feb",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25630228.subtract(weeks2days3),
  2563, 3, "M03", 16, "add 2w 3d to day before leap day",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  date25640226.subtract(years1months2days4),
  2565, 4, "M04", 30, "add 1y 2mo 4d with result in common-year April",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2566, monthCode: "M02", day: 26, calendar }, options).subtract(years1months2days4),
  2567, 4, "M04", 30, "add 1y 2mo 4d with result in leap-year April",
  "be", 2567);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M12", day: 30, calendar }, options).subtract(years1months2days4),
  2566, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into common-year March",
  "be", 2566);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2565, monthCode: "M12", day: 30, calendar }, options).subtract(years1months2days4),
  2567, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into leap-year March",
  "be", 2567);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2565, monthCode: "M12", day: 29, calendar }, options).subtract(years1months2days4),
  2567, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into leap-year March",
  "be", 2567);

TemporalHelpers.assertPlainDate(
  date25630303.subtract(days10n),
  2563, 2, "M02", 22, "subtract 10d crossing leap day",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25640303.subtract(days10n),
  2564, 2, "M02", 21, "subtract 10d crossing end of common-year Feb",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2563, monthCode: "M03", day: 10, calendar }, options).subtract(days10n),
  2563, 2, "M02", 29, "subtract 10d with result on leap day",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M03", day: 10, calendar }, options).subtract(days10n),
  2564, 2, "M02", 28, "subtract 10d with result on common-year Feb 28",
  "be", 2564);

TemporalHelpers.assertPlainDate(
  date25630229.subtract(weeks2days3n),
  2563, 2, "M02", 12, "subtract 2w 3d from leap day",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  date25640301.subtract(weeks2days3n),
  2564, 2, "M02", 12, "subtract 2w 3d from common-year Mar 1",
  "be", 2564);
TemporalHelpers.assertPlainDate(
  date25630301.subtract(weeks2days3n),
  2563, 2, "M02", 13, "subtract 2w 3d from leap-year Mar 1",
  "be", 2563);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2566, monthCode: "M03", day: 24, calendar }, options).subtract(years1months2days4n),
  2565, 1, "M01", 20, "subtract 1y 2mo 4d with result in common-year January",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M03", day: 24, calendar }, options).subtract(years1months2days4n),
  2563, 1, "M01", 20, "subtract 1y 2mo 4d with result in leap-year January",
  "be", 2563);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2566, monthCode: "M05", day: 1, calendar }, options).subtract(years1months2days4n),
  2565, 2, "M02", 25, "add 1y 2mo 4d with result rolling over into common-year February",
  "be", 2565);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 2564, monthCode: "M05", day: 1, calendar }, options).subtract(years1months2days4n),
  2563, 2, "M02", 26, "add 1y 2mo 4d with result rolling over into leap-year February",
  "be", 2563);
