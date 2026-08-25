// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Constraining the day for 29/30-day months in islamic-tbla calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-tbla";
const options = { overflow: "reject" };

// 30-day months: 01, 03, 05, 07, 09, 11
// 29-day months: 02, 04, 06, 08, 10
// Month 12 (Dhu al-Hijjah) has 29 days in common years and 30 in leap years.
// 1445 is a leap year, 1444 and 1446 are common years.

const date14440130 = Temporal.PlainDate.from({ year: 1444, monthCode: "M01", day: 30, calendar }, options);
const date14450130 = Temporal.PlainDate.from({ year: 1445, monthCode: "M01", day: 30, calendar }, options);
const date14460130 = Temporal.PlainDate.from({ year: 1446, monthCode: "M01", day: 30, calendar }, options);

const months1 = new Temporal.Duration(0, 1);
const months2 = new Temporal.Duration(0, 2);
const months3 = new Temporal.Duration(0, 3);
const months4 = new Temporal.Duration(0, 4);
const months5 = new Temporal.Duration(0, 5);
const months6 = new Temporal.Duration(0, 6);
const months7 = new Temporal.Duration(0, 7);
const months8 = new Temporal.Duration(0, 8);
const months9 = new Temporal.Duration(0, 9);
const months10 = new Temporal.Duration(0, 10);
const months11 = new Temporal.Duration(0, 11);
const months1n = new Temporal.Duration(0, -1);
const months2n = new Temporal.Duration(0, -2);
const months3n = new Temporal.Duration(0, -3);
const months4n = new Temporal.Duration(0, -4);
const months5n = new Temporal.Duration(0, -5);
const months6n = new Temporal.Duration(0, -6);
const months7n = new Temporal.Duration(0, -7);
const months8n = new Temporal.Duration(0, -8);
const months9n = new Temporal.Duration(0, -9);
const months10n = new Temporal.Duration(0, -10);
const months11n = new Temporal.Duration(0, -11);
const months12n = new Temporal.Duration(0, -12);

// Common year, forwards

TemporalHelpers.assertPlainDate(
  date14440130.add(months1),
  1444, 2, "M02", 29, "common-year Safar constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14440130.add(months1, options);
}, "common-year Safar rejects with 30");

TemporalHelpers.assertPlainDate(
  date14440130.add(months2, options),
  1444, 3, "M03", 30, "common-year Rabi' al-Awwal does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14440130.add(months3),
  1444, 4, "M04", 29, "common-year Rabi' al-Thani constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14440130.add(months3, options);
}, "common-year Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDate(
  date14440130.add(months4, options),
  1444, 5, "M05", 30, "common-year Jumada al-Awwal does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14440130.add(months5),
  1444, 6, "M06", 29, "common-year Jumada al-Thani constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14440130.add(months5, options);
}, "common-year Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDate(
  date14440130.add(months6, options),
  1444, 7, "M07", 30, "common-year Rajab does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14440130.add(months7),
  1444, 8, "M08", 29, "common-year Sha'ban constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14440130.add(months7, options);
}, "common-year Sha'ban rejects with 30");

TemporalHelpers.assertPlainDate(
  date14440130.add(months8, options),
  1444, 9, "M09", 30, "common-year Ramadan does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14440130.add(months9),
  1444, 10, "M10", 29, "common-year Shawwal constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14440130.add(months9, options);
}, "common-year Shawwal rejects with 30");

TemporalHelpers.assertPlainDate(
  date14440130.add(months10, options),
  1444, 11, "M11", 30, "common-year Dhu al-Qadah does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14440130.add(months11),
  1444, 12, "M12", 29, "common-year Dhu al-Hijjah constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14440130.add(months11, options);
}, "common-year Dhu al-Hijjah rejects with 30");

// Leap year, forwards

TemporalHelpers.assertPlainDate(
  date14450130.add(months1),
  1445, 2, "M02", 29, "leap-year Safar constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14450130.add(months1, options);
}, "leap-year Safar rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months2, options),
  1445, 3, "M03", 30, "leap-year Rabi' al-Awwal does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14450130.add(months3),
  1445, 4, "M04", 29, "leap-year Rabi' al-Thani constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14450130.add(months3, options);
}, "leap-year Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months4, options),
  1445, 5, "M05", 30, "leap-year Jumada al-Awwal does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14450130.add(months5),
  1445, 6, "M06", 29, "leap-year Jumada al-Thani constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14450130.add(months5, options);
}, "leap-year Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months6, options),
  1445, 7, "M07", 30, "leap-year Rajab does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14450130.add(months7),
  1445, 8, "M08", 29, "leap-year Sha'ban constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14450130.add(months7, options);
}, "leap-year Sha'ban rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months8, options),
  1445, 9, "M09", 30, "leap-year Ramadan does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14450130.add(months9),
  1445, 10, "M10", 29, "leap-year Shawwal constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14450130.add(months9, options);
}, "leap-year Shawwal rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months10, options),
  1445, 11, "M11", 30, "leap-year Dhu al-Qadah does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14450130.add(months11, options),
  1445, 12, "M12", 30, "leap-year Dhu al-Hijjah does not reject 30",
  "ah", 1445);

// Common year, backwards

TemporalHelpers.assertPlainDate(
  date14450130.add(months12n, options),
  1444, 1, "M01", 30, "common-year Muharram does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14450130.add(months11n),
  1444, 2, "M02", 29, "common-year Safar constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14450130.add(months11n, options);
}, "common-year Safar rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months10n, options),
  1444, 3, "M03", 30, "common-year Rabi' al-Awwal does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14450130.add(months9n),
  1444, 4, "M04", 29, "common-year Rabi' al-Thani constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14450130.add(months9n, options);
}, "common-year Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months8n, options),
  1444, 5, "M05", 30, "common-year Jumada al-Awwal does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14450130.add(months7n),
  1444, 6, "M06", 29, "common-year Jumada al-Thani constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14450130.add(months7n, options);
}, "common-year Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months6n, options),
  1444, 7, "M07", 30, "common-year Rajab does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14450130.add(months5n),
  1444, 8, "M08", 29, "common-year Sha'ban constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14450130.add(months5n, options);
}, "common-year Sha'ban rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months4n, options),
  1444, 9, "M09", 30, "common-year Ramadan does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14450130.add(months3n),
  1444, 10, "M10", 29, "common-year Shawwal constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14450130.add(months3n, options);
}, "common-year Shawwal rejects with 30");

TemporalHelpers.assertPlainDate(
  date14450130.add(months2n, options),
  1444, 11, "M11", 30, "common-year Dhu al-Qadah does not reject 30",
  "ah", 1444);

TemporalHelpers.assertPlainDate(
  date14450130.add(months1n),
  1444, 12, "M12", 29, "common-year Dhu al-Hijjah constrains to 29",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14450130.add(months1n, options);
}, "common-year Dhu al-Hijjah rejects with 30");

// Leap year, backwards

TemporalHelpers.assertPlainDate(
  date14460130.add(months12n, options),
  1445, 1, "M01", 30, "leap-year Muharram does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14460130.add(months11n),
  1445, 2, "M02", 29, "leap-year Safar constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14460130.add(months11n, options);
}, "leap-year Safar rejects with 30");

TemporalHelpers.assertPlainDate(
  date14460130.add(months10n, options),
  1445, 3, "M03", 30, "leap-year Rabi' al-Awwal does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14460130.add(months9n),
  1445, 4, "M04", 29, "leap-year Rabi' al-Thani constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14460130.add(months9n, options);
}, "leap-year Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDate(
  date14460130.add(months8n, options),
  1445, 5, "M05", 30, "leap-year Jumada al-Awwal does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14460130.add(months7n),
  1445, 6, "M06", 29, "leap-year Jumada al-Thani constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14460130.add(months7n, options);
}, "leap-year Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDate(
  date14460130.add(months6n, options),
  1445, 7, "M07", 30, "leap-year Rajab does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14460130.add(months5n),
  1445, 8, "M08", 29, "leap-year Sha'ban constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14460130.add(months5n, options);
}, "leap-year Sha'ban rejects with 30");

TemporalHelpers.assertPlainDate(
  date14460130.add(months4n, options),
  1445, 9, "M09", 30, "leap-year Ramadan does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14460130.add(months3n),
  1445, 10, "M10", 29, "leap-year Shawwal constrains to 29",
  "ah", 1445);
assert.throws(RangeError, function () {
  date14460130.add(months3n, options);
}, "leap-year Shawwal rejects with 30");

TemporalHelpers.assertPlainDate(
  date14460130.add(months2n, options),
  1445, 11, "M11", 30, "leap-year Dhu al-Qadah does not reject 30",
  "ah", 1445);

TemporalHelpers.assertPlainDate(
  date14460130.add(months1n, options),
  1445, 12, "M12", 30, "leap-year Dhu al-Hijjah does not reject 30",
  "ah", 1445);
