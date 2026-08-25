// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Check various basic calculations involving leap years (Persian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "persian";
const options = { overflow: "reject" };

// Years

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

const date13621230 = Temporal.PlainDateTime.from({ year: 1362, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date13621230.add(years1),
  1363, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "add 1y to leap day and constrain",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13621230.add(years1, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDateTime(
  date13621230.add(years4, options),
  1366, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "add 4y to leap day",
  "ap", 1366);

TemporalHelpers.assertPlainDateTime(
  date13621230.add(years1n),
  1361, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "subtract 1y from leap day and constrain",
  "ap", 1361);
assert.throws(RangeError, function () {
  date13621230.add(years1n, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDateTime(
  date13621230.add(years4n, options),
  1358, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "subtract 4y from leap day",
  "ap", 1358);

// Months

const months1 = new Temporal.Duration(0, 1);
const months1n = new Temporal.Duration(0, -1);
const months6 = new Temporal.Duration(0, 6);
const months5 = new Temporal.Duration(0, 5);
const months11n = new Temporal.Duration(0, -11);
const years1months2 = new Temporal.Duration(1, 2);
const years1months2n = new Temporal.Duration(-1, -2);

const date13620631 = Temporal.PlainDateTime.from({ year: 1362, monthCode: "M06", day: 31, hour: 12, minute: 34, calendar }, options);
const date13621130 = Temporal.PlainDateTime.from({ year: 1362, monthCode: "M11", day: 30, hour: 12, minute: 34, calendar }, options);
const date13630131 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M01", day: 31, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date13620631.add(months6),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "add 6mo to Shahrivar 31 constrains to Esfand 30 in leap year",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620631.add(months6, options);
}, "add 6mo to Shahrivar 31 rejects");

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1362, monthCode: "M10", day: 30, hour: 12, minute: 34, calendar }, options).add(years1months2),
  1363, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the next year constrained to Esfand 29",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1361, monthCode: "M10", day: 30, hour: 12, minute: 34, calendar }, options).add(years1months2),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the next year constrained to Esfand 30",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13630131.add(months1n),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "subtract 1mo from Farvardin 31 constrains to Esfand 30 in leap year",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.add(months1n, options);
}, "subtract 1mo from Farvardin 31 rejects");

TemporalHelpers.assertPlainDateTime(
  date13621130.add(months11n),
  1361, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "subtract 11mo with result in the previous year constrained to Esfand 29",
  "ap", 1361);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1363, monthCode: "M11", day: 30, hour: 12, minute: 34, calendar }, options).add(months11n),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "subtract 11mo with result in the previous year constrained to Esfand 30",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1364, monthCode: "M02", day: 31, hour: 12, minute: 34, calendar }, options).add(years1months2n),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the previous year constrained to Esfand 30",
  "ap", 1362);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1365, monthCode: "M02", day: 31, hour: 12, minute: 34, calendar }, options).add(years1months2n),
  1363, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "add 1y 2mo with result in the previous year constrained to Esfand 29",
  "ap", 1363);

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

const date13610301 = Temporal.PlainDateTime.from({ year: 1361, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
const date13621128 = Temporal.PlainDateTime.from({ year: 1362, monthCode: "M11", day: 28, hour: 12, minute: 34, calendar }, options);
const date13621219 = Temporal.PlainDateTime.from({ year: 1362, monthCode: "M12", day: 19, hour: 12, minute: 34, calendar }, options);
const date13621229 = Temporal.PlainDateTime.from({ year: 1362, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar }, options);
const date13630101 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M01", day: 3, hour: 12, minute: 34, calendar }, options);
const date13630103 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M01", day: 3, hour: 12, minute: 34, calendar }, options);
const date13630201 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }, options);
const date13631128 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M11", day: 28, hour: 12, minute: 34, calendar }, options);
const date13631219 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M12", day: 19, hour: 12, minute: 34, calendar }, options);
const date13631229 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar }, options);
const date13640103 = Temporal.PlainDateTime.from({ year: 1364, monthCode: "M01", day: 3, hour: 12, minute: 34, calendar }, options);
const date13641229 = Temporal.PlainDateTime.from({ year: 1364, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1363, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar }, options).add(weeks1),
  1364, 1, "M01", 6, 12, 34, 0, 0, 0, 0, "add 1w in Esfand with result in Farvardin",
  "ap", 1364);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1362, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar }, options).add(weeks1),
  1363, 1, "M01", 5, 12, 34, 0, 0, 0, 0, "add 1w in Esfand with result in Farvardin in a leap year",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13631219.add(weeks6),
  1364, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "add 6w in Esfand with result in the next month",
  "ap", 1364);
TemporalHelpers.assertPlainDateTime(
  date13621219.add(weeks6),
  1363, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "add 6w in Esfand with result in the next month in a leap year",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1362, monthCode: "M11", day: 27, hour: 12, minute: 34, calendar }, options).add(weeks6),
  1363, 1, "M01", 9, 12, 34, 0, 0, 0, 0, "add 6w with result in the next year, crossing leap day",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13621229.add(years1weeks2),
  1364, 1, "M01", 14, 12, 34, 0, 0, 0, 0, "add 1y 2w to Esfand 29 with result in Farvardin starting in leap year",
  "ap", 1364);
TemporalHelpers.assertPlainDateTime(
  date13631229.add(years1weeks2),
  1365, 1, "M01", 14, 12, 34, 0, 0, 0, 0, "add 1y 2w to Esfand 29 with result in Farvardin starting in common year",
  "ap", 1365);
TemporalHelpers.assertPlainDateTime(
  date13621230.add(years1weeks2),
  1364, 1, "M01", 14, 12, 34, 0, 0, 0, 0, "add 1y 2w to Esfand 30 with result in Farvardin",
  "ap", 1364);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1361, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar }, options).add(years1weeks2),
  1363, 1, "M01", 12, 12, 34, 0, 0, 0, 0, "add 1y 2w to Esfand 28 with result in Farvardin crossing leap year",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13621230.add(months2weeks3),
  1363, 3, "M03", 20, 12, 34, 0, 0, 0, 0, "add 2mo 3w to leap day",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  date13621229.add(months2weeks3),
  1363, 3, "M03", 19, 12, 34, 0, 0, 0, 0, "add 2mo 3w to Esfand 29 of a leap year",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  date13641229.add(months2weeks3),
  1365, 3, "M03", 19, 12, 34, 0, 0, 0, 0, "add 2mo 3w to Esfand 28 of a common year",
  "ap", 1365);

TemporalHelpers.assertPlainDateTime(
  date13640103.add(weeks1n),
  1363, 12, "M12", 25, 12, 34, 0, 0, 0, 0, "subtract 1w in Farvardin with result in Esfand",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  date13630103.add(weeks1n),
  1362, 12, "M12", 26, 12, 34, 0, 0, 0, 0, "subtract 1w in Farvardin with result in leap-year Esfand",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1364, monthCode: "M02", day: 2, hour: 12, minute: 34, calendar }, options).add(weeks6n),
  1363, 12, "M12", 20, 12, 34, 0, 0, 0, 0, "subtract 6w with result in Esfand",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1363, monthCode: "M02", day: 2, hour: 12, minute: 34, calendar }, options).add(weeks6n),
  1362, 12, "M12", 21, 12, 34, 0, 0, 0, 0, "subtract 6w with result in leap-year Esfand",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13621230.add(years1weeks2n),
  1361, 12, "M12", 15, 12, 34, 0, 0, 0, 0, "subtract 1y 2w from Esfand 30 starting in leap year",
  "ap", 1361);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1363, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar }, options).add(years1weeks2n),
  1362, 12, "M12", 15, 12, 34, 0, 0, 0, 0, "subtract 1y 2w from Esfand 29 starting in common year",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13621230.add(months2weeks3n),
  1362, 10, "M10", 9, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w from leap day",
  "ap", 1362);
TemporalHelpers.assertPlainDateTime(
  date13630101.add(months2weeks3n),
  1362, 10, "M10", 12, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w from Farvardin 1, ending in a leap year",
  "ap", 1362);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1362, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options).add(months2weeks3n),
  1361, 10, "M10", 10, 12, 34, 0, 0, 0, 0, "subtract 2mo 3w from Farvardin 1, ending in a common year",
  "ap", 1361);
TemporalHelpers.assertPlainDateTime(
  date13610301.add(months11weeks3n),
  1360, 3, "M03", 11, 12, 34, 0, 0, 0, 0, "subtract 11mo 3w from Khordad 1 crossing common-year Esfand",
  "ap", 1360);
TemporalHelpers.assertPlainDateTime(
  date13630201.add(months11weeks3n),
  1362, 2, "M02", 11, 12, 34, 0, 0, 0, 0, "subtract 11mo 3w from Ordibehesht 1 crossing leap-year Esfand",
  "ap", 1362);

// Days

const days10 = new Temporal.Duration(0, 0, 0, 10);
const days10n = new Temporal.Duration(0, 0, 0, -10);
const weeks2days3 = new Temporal.Duration(0, 0, 2, 3);
const weeks2days3n = new Temporal.Duration(0, 0, -2, -3);
const years1months2days4 = new Temporal.Duration(1, 2, 0, 4);
const years1months2days4n = new Temporal.Duration(-1, -2, 0, -4);

const date13621220 = Temporal.PlainDateTime.from({ year: 1362, monthCode: "M12", day: 20, hour: 12, minute: 34, calendar }, options);
const date13631225 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M12", day: 25, hour: 12, minute: 34, calendar }, options);
const date13631227 = Temporal.PlainDateTime.from({ year: 1363, monthCode: "M12", day: 27, hour: 12, minute: 34, calendar }, options);
const date13640101 = Temporal.PlainDateTime.from({ year: 1364, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);
const date13641219 = Temporal.PlainDateTime.from({ year: 1364, monthCode: "M12", day: 19, hour: 12, minute: 34, calendar }, options);
const date13641220 = Temporal.PlainDateTime.from({ year: 1364, monthCode: "M12", day: 20, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1362, monthCode: "M12", day: 25, hour: 12, minute: 34, calendar }, options).add(days10),
  1363, 1, "M01", 5, 12, 34, 0, 0, 0, 0, "add 10d crossing leap day",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  date13631225.add(days10),
  1364, 1, "M01", 6, 12, 34, 0, 0, 0, 0, "add 10d crossing end of common-year Esfand",
  "ap", 1364);
TemporalHelpers.assertPlainDateTime(
  date13621220.add(days10),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "add 10d with result on leap day",
  "ap", 1362);
TemporalHelpers.assertPlainDateTime(
  date13641220.add(days10),
  1365, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "add 10d with result on common-year Farvardin 1",
  "ap", 1365);

TemporalHelpers.assertPlainDateTime(
  date13621230.add(weeks2days3),
  1363, 1, "M01", 17, 12, 34, 0, 0, 0, 0, "add 2w 3d to leap day",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  date13631229.add(weeks2days3),
  1364, 1, "M01", 17, 12, 34, 0, 0, 0, 0, "add 2w 3d to end of common-year Esfand",
  "ap", 1364);
TemporalHelpers.assertPlainDateTime(
  date13621229.add(weeks2days3),
  1363, 1, "M01", 16, 12, 34, 0, 0, 0, 0, "add 2w 3d to day before leap day",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13631227.add(years1months2days4),
  1365, 2, "M02", 31, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result in common-year Ordibehesht",
  "ap", 1365);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1362, monthCode: "M12", day: 26, hour: 12, minute: 34, calendar }, options).add(years1months2days4),
  1364, 2, "M02", 30, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d crossing leap-year Esfand",
  "ap", 1364);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1362, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar }, options).add(years1months2days4),
  1364, 3, "M03", 3, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into common-year Khordad",
  "ap", 1364);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1361, monthCode: "M10", day: 27, hour: 12, minute: 34, calendar }, options).add(years1months2days4),
  1363, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into Farvardin immediately after leap year",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13630103.add(days10n),
  1362, 12, "M12", 23, 12, 34, 0, 0, 0, 0, "subtract 10d crossing leap day",
  "ap", 1362);
TemporalHelpers.assertPlainDateTime(
  date13640103.add(days10n),
  1363, 12, "M12", 22, 12, 34, 0, 0, 0, 0, "subtract 10d crossing end of common-year Esfand",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1363, monthCode: "M01", day: 10, hour: 12, minute: 34, calendar }, options).add(days10n),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "subtract 10d with result on leap day",
  "ap", 1362);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1364, monthCode: "M01", day: 10, hour: 12, minute: 34, calendar }, options).add(days10n),
  1363, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "subtract 10d with result on common-year Esfand 29",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13621230.add(weeks2days3n),
  1362, 12, "M12", 13, 12, 34, 0, 0, 0, 0, "subtract 2w 3d from leap day",
  "ap", 1362);
TemporalHelpers.assertPlainDateTime(
  date13640101.add(weeks2days3n),
  1363, 12, "M12", 13, 12, 34, 0, 0, 0, 0, "subtract 2w 3d from Farvardin 1 following a common year",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  date13630101.add(weeks2days3n),
  1362, 12, "M12", 16, 12, 34, 0, 0, 0, 0, "subtract 2w 3d from Farvardin 1 following a leap year",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1365, monthCode: "M01", day: 24, hour: 12, minute: 34, calendar }, options).add(years1months2days4n),
  1363, 11, "M11", 20, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in common-year Bahman",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1364, monthCode: "M01", day: 24, hour: 12, minute: 34, calendar }, options).add(years1months2days4n),
  1362, 11, "M11", 20, 12, 34, 0, 0, 0, 0, "subtract 1y 2mo 4d with result in leap-year Bahman",
  "ap", 1362);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1365, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options).add(years1months2days4n),
  1363, 12, "M12", 26, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into common-year Esfand",
  "ap", 1363);
TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({ year: 1364, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options).add(years1months2days4n),
  1362, 12, "M12", 27, 12, 34, 0, 0, 0, 0, "add 1y 2mo 4d with result rolling over into leap-year Esfand",
  "ap", 1362);
