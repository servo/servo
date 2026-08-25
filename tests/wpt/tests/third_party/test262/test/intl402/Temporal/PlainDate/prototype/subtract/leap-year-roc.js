// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Check various basic calculations involving leap years (roc calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

const date1090229 = Temporal.PlainDate.from({ year: 109, monthCode: "M02", day: 29, calendar }, options);

TemporalHelpers.assertPlainDate(
  date1090229.subtract(years1),
  110, 2, "M02", 28, "add 1y to leap day and constrain",
  "roc", 110);
assert.throws(RangeError, function () {
  date1090229.subtract(years1, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDate(
  date1090229.subtract(years4, options),
  113, 2, "M02", 29, "add 4y to leap day",
  "roc", 113);

TemporalHelpers.assertPlainDate(
  date1090229.subtract(years1n),
  108, 2, "M02", 28, "subtract 1y from leap day and constrain",
  "roc", 108);
assert.throws(RangeError, function () {
  date1090229.subtract(years1n, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDate(
  date1090229.subtract(years4n, options),
  105, 2, "M02", 29, "subtract 4y from leap day",
  "roc", 105);

// Months

const months1 = new Temporal.Duration(0, -1);
const months1n = new Temporal.Duration(0, 1);
const months5 = new Temporal.Duration(0, -5);
const months11n = new Temporal.Duration(0, 11);
const years1months2 = new Temporal.Duration(-1, -2);
const years1months2n = new Temporal.Duration(1, 2);

const date1090131 = Temporal.PlainDate.from({ year: 109, monthCode: "M01", day: 31, calendar }, options);
const date1090331 = Temporal.PlainDate.from({ year: 109, monthCode: "M03", day: 31, calendar }, options);

TemporalHelpers.assertPlainDate(
  date1090131.subtract(months1),
  109, 2, "M02", 29, "add 1mo to Jan 31 constrains to Feb 29 in leap year",
  "roc", 109);
assert.throws(RangeError, function () {
  date1090131.subtract(months1, options);
}, "add 1mo to Jan 31 rejects");

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M09", day: 30, calendar }, options).subtract(months5),
  111, 2, "M02", 28, "add 5mo with result in the next year constrained to Feb 28",
  "roc", 111);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 108, monthCode: "M09", day: 30, calendar }, options).subtract(months5),
  109, 2, "M02", 29, "add 5mo with result in the next year constrained to Feb 29",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M12", day: 31, calendar }, options).subtract(years1months2),
  112, 2, "M02", 28, "add 1y 2mo with result in the next year constrained to Feb 28",
  "roc", 112);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 111, monthCode: "M12", day: 31, calendar }, options).subtract(years1months2),
  113, 2, "M02", 29, "add 1y 2mo with result in the next year constrained to Feb 29",
  "roc", 113);

TemporalHelpers.assertPlainDate(
  date1090331.subtract(months1n),
  109, 2, "M02", 29, "subtract 1mo from Mar 31 constrains to Feb 29 in leap year",
  "roc", 109);
assert.throws(RangeError, function () {
  date1090331.subtract(months1n, options);
}, "subtract 1mo from Mar 31 rejects");

TemporalHelpers.assertPlainDate(
  date1090131.subtract(months11n),
  108, 2, "M02", 28, "subtract 11mo with result in the previous year constrained to Feb 28",
  "roc", 108);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M01", day: 31, calendar }, options).subtract(months11n),
  109, 2, "M02", 29, "add 11mo with result in the next year constrained to Feb 29",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 111, monthCode: "M04", day: 30, calendar }, options).subtract(years1months2n),
  110, 2, "M02", 28, "add 1y 2mo with result in the previous year constrained to Feb 28",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M04", day: 30, calendar }, options).subtract(years1months2n),
  109, 2, "M02", 29, "add 1y 2mo with result in the previous year constrained to Feb 29",
  "roc", 109);

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

const date1081228 = Temporal.PlainDate.from({ year: 108, monthCode: "M12", day: 28, calendar }, options);
const date1090219 = Temporal.PlainDate.from({ year: 109, monthCode: "M02", day: 19, calendar }, options);
const date1090228 = Temporal.PlainDate.from({ year: 109, monthCode: "M02", day: 28, calendar }, options);
const date1090301 = Temporal.PlainDate.from({ year: 109, monthCode: "M03", day: 1, calendar }, options);
const date1090303 = Temporal.PlainDate.from({ year: 109, monthCode: "M03", day: 3, calendar }, options);
const date1091228 = Temporal.PlainDate.from({ year: 109, monthCode: "M12", day: 28, calendar }, options);
const date1100219 = Temporal.PlainDate.from({ year: 110, monthCode: "M02", day: 19, calendar }, options);
const date1100228 = Temporal.PlainDate.from({ year: 110, monthCode: "M02", day: 28, calendar }, options);
const date1100301 = Temporal.PlainDate.from({ year: 110, monthCode: "M03", day: 1, calendar }, options);
const date1100303 = Temporal.PlainDate.from({ year: 110, monthCode: "M03", day: 3, calendar }, options);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M02", day: 27, calendar }, options).subtract(weeks1),
  110, 3, "M03", 6, "add 1w in Feb with result in March",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 109, monthCode: "M02", day: 27, calendar }, options).subtract(weeks1),
  109, 3, "M03", 5, "add 1w in Feb with result in March in a leap year",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  date1100219.subtract(weeks6),
  110, 4, "M04", 2, "add 6w in Feb with result in the next month",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  date1090219.subtract(weeks6),
  109, 4, "M04", 1, "add 6w in Feb with result in the next month in a leap year",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 109, monthCode: "M01", day: 27, calendar }, options).subtract(weeks6),
  109, 3, "M03", 9, "add 6w with result in the same year, crossing leap day",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  date1090228.subtract(years1weeks2),
  110, 3, "M03", 14, "add 1y 2w to Feb 28 with result in March starting in leap year",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  date1100228.subtract(years1weeks2),
  111, 3, "M03", 14, "add 1y 2w to Feb 28 with result in March starting in common year",
  "roc", 111);
TemporalHelpers.assertPlainDate(
  date1090229.subtract(years1weeks2),
  110, 3, "M03", 14, "add 1y 2w to Feb 29 with result in March",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 108, monthCode: "M02", day: 28, calendar }, options).subtract(years1weeks2),
  109, 3, "M03", 13, "add 1y 2w to Feb 28 with result in March ending in leap year",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  date1090229.subtract(months2weeks3),
  109, 5, "M05", 20, "add 2mo 3w to leap day",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  date1090228.subtract(months2weeks3),
  109, 5, "M05", 19, "add 2mo 3w to Feb 28 of a leap year",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  date1100228.subtract(months2weeks3),
  110, 5, "M05", 19, "add 2mo 3w to Feb 28 of a common year",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  date1091228.subtract(months2weeks3),
  110, 3, "M03", 21, "add 2mo 3w from end of year crossing common-year Feb",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  date1081228.subtract(months2weeks3),
  109, 3, "M03", 20, "add 2mo 3w from end of year crossing leap-year Feb",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  date1100303.subtract(weeks1n),
  110, 2, "M02", 24, "subtract 1w in March with result in Feb",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  date1090303.subtract(weeks1n),
  109, 2, "M02", 25, "subtract 1w in March with result in leap-year Feb",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M04", day: 2, calendar }, options).subtract(weeks6n),
  110, 2, "M02", 19, "subtract 6w with result in Feb",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 109, monthCode: "M04", day: 2, calendar }, options).subtract(weeks6n),
  109, 2, "M02", 20, "subtract 6w with result in leap-year Feb",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 109, monthCode: "M03", day: 9, calendar }, options).subtract(weeks6n),
  109, 1, "M01", 27, "subtract 6w with result in the same year, crossing leap day",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  date1090301.subtract(years1weeks2n),
  108, 2, "M02", 15, "subtract 1y 2w from Mar 1 starting in leap year",
  "roc", 108);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 112, monthCode: "M03", day: 1, calendar }, options).subtract(years1weeks2n),
  111, 2, "M02", 15, "subtract 1y 2w from Mar 1 starting in common year",
  "roc", 111);
TemporalHelpers.assertPlainDate(
  date1090229.subtract(years1weeks2n),
  108, 2, "M02", 14, "subtract 1y 2w from Feb 29",
  "roc", 108);
TemporalHelpers.assertPlainDate(
  date1100301.subtract(years1weeks2n),
  109, 2, "M02", 16, "subtract 1y 2w from Mar 1 ending in leap year",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  date1090229.subtract(months2weeks3n),
  108, 12, "M12", 8, "subtract 2mo 3w from leap day",
  "roc", 108);
TemporalHelpers.assertPlainDate(
  date1090301.subtract(months2weeks3n),
  108, 12, "M12", 11, "subtract 2mo 3w from Mar 1 of a leap year",
  "roc", 108);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 108, monthCode: "M03", day: 1, calendar }, options).subtract(months2weeks3n),
  107, 12, "M12", 11, "subtract 2mo 3w from Mar 1 of a common year",
  "roc", 107);
TemporalHelpers.assertPlainDate(
  date1081228.subtract(months11weeks3n),
  108, 1, "M01", 7, "add 2mo 3w from end of year crossing common-year Feb",
  "roc", 108);
TemporalHelpers.assertPlainDate(
  date1091228.subtract(months11weeks3n),
  109, 1, "M01", 7, "add 2mo 3w from end of year crossing leap-year Feb",
  "roc", 109);

// Days

const days10 = new Temporal.Duration(0, 0, 0, -10);
const days10n = new Temporal.Duration(0, 0, 0, 10);
const weeks2days3 = new Temporal.Duration(0, 0, -2, -3);
const weeks2days3n = new Temporal.Duration(0, 0, 2, 3);
const years1months2days4 = new Temporal.Duration(-1, -2, 0, -4);
const years1months2days4n = new Temporal.Duration(1, 2, 0, 4);

const date1100226 = Temporal.PlainDate.from({ year: 110, monthCode: "M02", day: 26, calendar }, options);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 109, monthCode: "M02", day: 26, calendar }, options).subtract(days10),
  109, 3, "M03", 7, "add 10d crossing leap day",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  date1100226.subtract(days10),
  110, 3, "M03", 8, "add 10d crossing end of common-year Feb",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  date1090219.subtract(days10),
  109, 2, "M02", 29, "add 10d with result on leap day",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  date1100219.subtract(days10),
  110, 3, "M03", 1, "add 10d with result on common-year March 1",
  "roc", 110);

TemporalHelpers.assertPlainDate(
  date1090229.subtract(weeks2days3),
  109, 3, "M03", 17, "add 2w 3d to leap day",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  date1100228.subtract(weeks2days3),
  110, 3, "M03", 17, "add 2w 3d to end of common-year Feb",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  date1090228.subtract(weeks2days3),
  109, 3, "M03", 16, "add 2w 3d to day before leap day",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  date1100226.subtract(years1months2days4),
  111, 4, "M04", 30, "add 1y 2mo 4d with result in common-year April",
  "roc", 111);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 112, monthCode: "M02", day: 26, calendar }, options).subtract(years1months2days4),
  113, 4, "M04", 30, "add 1y 2mo 4d with result in leap-year April",
  "roc", 113);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M12", day: 30, calendar }, options).subtract(years1months2days4),
  112, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into common-year March",
  "roc", 112);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 111, monthCode: "M12", day: 30, calendar }, options).subtract(years1months2days4),
  113, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into leap-year March",
  "roc", 113);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 111, monthCode: "M12", day: 29, calendar }, options).subtract(years1months2days4),
  113, 3, "M03", 4, "add 1y 2mo 4d with result rolling over into leap-year March",
  "roc", 113);

TemporalHelpers.assertPlainDate(
  date1090303.subtract(days10n),
  109, 2, "M02", 22, "subtract 10d crossing leap day",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  date1100303.subtract(days10n),
  110, 2, "M02", 21, "subtract 10d crossing end of common-year Feb",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 109, monthCode: "M03", day: 10, calendar }, options).subtract(days10n),
  109, 2, "M02", 29, "subtract 10d with result on leap day",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M03", day: 10, calendar }, options).subtract(days10n),
  110, 2, "M02", 28, "subtract 10d with result on common-year Feb 28",
  "roc", 110);

TemporalHelpers.assertPlainDate(
  date1090229.subtract(weeks2days3n),
  109, 2, "M02", 12, "subtract 2w 3d from leap day",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  date1100301.subtract(weeks2days3n),
  110, 2, "M02", 12, "subtract 2w 3d from common-year Mar 1",
  "roc", 110);
TemporalHelpers.assertPlainDate(
  date1090301.subtract(weeks2days3n),
  109, 2, "M02", 13, "subtract 2w 3d from leap-year Mar 1",
  "roc", 109);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 112, monthCode: "M03", day: 24, calendar }, options).subtract(years1months2days4n),
  111, 1, "M01", 20, "subtract 1y 2mo 4d with result in common-year January",
  "roc", 111);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M03", day: 24, calendar }, options).subtract(years1months2days4n),
  109, 1, "M01", 20, "subtract 1y 2mo 4d with result in leap-year January",
  "roc", 109);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 112, monthCode: "M05", day: 1, calendar }, options).subtract(years1months2days4n),
  111, 2, "M02", 25, "add 1y 2mo 4d with result rolling over into common-year February",
  "roc", 111);
TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 110, monthCode: "M05", day: 1, calendar }, options).subtract(years1months2days4n),
  109, 2, "M02", 26, "add 1y 2mo 4d with result rolling over into leap-year February",
  "roc", 109);
