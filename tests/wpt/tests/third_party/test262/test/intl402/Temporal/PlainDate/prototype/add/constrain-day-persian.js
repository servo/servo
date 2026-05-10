// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Constraining the day for 30/31-day months in Persian calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "persian";
const options = { overflow: "reject" };

// 31-day months: 01, 02, 03, 04, 05, 06
// 30-day months: 07, 08, 09, 10, 11
// Month 12 (Esfand) has 29 days in common years and 30 in leap years.
// 1362 is a leap year, 1363 and 1364 are common years.

const date13620131 = Temporal.PlainDate.from({ year: 1362, monthCode: "M01", day: 31, calendar }, options);
const date13630131 = Temporal.PlainDate.from({ year: 1363, monthCode: "M01", day: 31, calendar }, options);
const date13640131 = Temporal.PlainDate.from({ year: 1364, monthCode: "M01", day: 31, calendar }, options);

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
  date13630131.add(months6),
  1363, 7, "M07", 30, "common-year Mehr constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.add(months6, options);
}, "common-year Mehr rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months1, options),
  1363, 2, "M02", 31, "common-year Ordibehesht does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13630131.add(months7),
  1363, 8, "M08", 30, "common-year Aban constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.add(months7, options);
}, "common-year Aban rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months2, options),
  1363, 3, "M03", 31, "common-year Khordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13630131.add(months8),
  1363, 9, "M09", 30, "common-year Azar constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13620131.add(months8, options);
}, "common-year Azar rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months3, options),
  1363, 4, "M04", 31, "common-year Tir does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13630131.add(months9),
  1363, 10, "M10", 30, "common-year Dey constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.add(months9, options);
}, "common-year Dey rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months4, options),
  1363, 5, "M05", 31, "common-year Mordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13630131.add(months10),
  1363, 11, "M11", 30, "common-year Bahman constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.add(months10, options);
}, "common-year Bahman rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months5, options),
  1363, 6, "M06", 31, "common-year Shahrivar does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13630131.add(months11),
  1363, 12, "M12", 29, "common-year Esfand constrains to 29",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.add(months11, options);
}, "common-year Esfand rejects with 31");

// Leap year, forwards

TemporalHelpers.assertPlainDate(
  date13620131.add(months6),
  1362, 7, "M07", 30, "leap-year Mehr constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.add(months6, options);
}, "leap-year Mehr rejects with 31");

TemporalHelpers.assertPlainDate(
  date13620131.add(months1, options),
  1362, 2, "M02", 31, "leap-year Ordibehesht does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13620131.add(months7),
  1362, 8, "M08", 30, "leap-year Aban constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.add(months7, options);
}, "leap-year Aban rejects with 31");

TemporalHelpers.assertPlainDate(
  date13620131.add(months2, options),
  1362, 3, "M03", 31, "leap-year Khordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13620131.add(months8),
  1362, 9, "M09", 30, "leap-year Azar constrains to 29",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.add(months8, options);
}, "leap-year Azar rejects with 31");

TemporalHelpers.assertPlainDate(
  date13620131.add(months3, options),
  1362, 4, "M04", 31, "leap-year Tir does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13620131.add(months9),
  1362, 10, "M10", 30, "leap-year Dey constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.add(months9, options);
}, "leap-year Dey rejects with 31");

TemporalHelpers.assertPlainDate(
  date13620131.add(months4, options),
  1362, 5, "M05", 31, "leap-year Mordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13620131.add(months10),
  1362, 11, "M11", 30, "leap-year Bahman constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.add(months10, options);
}, "leap-year Bahman rejects with 31");

TemporalHelpers.assertPlainDate(
  date13620131.add(months5, options),
  1362, 6, "M06", 31, "leap-year Shahrivar does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13620131.add(months11),
  1362, 12, "M12", 30, "leap-year Esfand constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.add(months11, options);
}, "leap-year Esfand rejects with 30");

// Common year, backwards

TemporalHelpers.assertPlainDate(
  date13640131.add(months12n, options),
  1363, 1, "M01", 31, "common-year Farvardin does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13640131.add(months11n, options),
  1363, 2, "M02", 31, "common-year Ordibehesht does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13640131.add(months10n, options),
  1363, 3, "M03", 31, "common-year Khordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13640131.add(months9n, options),
  1363, 4, "M04", 31, "common-year Tir does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13640131.add(months8n, options),
  1363, 5, "M05", 31, "common-year Mordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13640131.add(months7n, options),
  1363, 6, "M06", 31, "common-year Shahrivar does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  date13640131.add(months6n),
  1363, 7, "M07", 30, "common-year Mehr constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.add(months6n, options);
}, "common-year Mehr rejects with 31");

TemporalHelpers.assertPlainDate(
  date13640131.add(months5n),
  1363, 8, "M08", 30, "common-year Aban constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.add(months5n, options);
}, "common-year Aban rejects with 31");

TemporalHelpers.assertPlainDate(
  date13640131.add(months4n),
  1363, 9, "M09", 30, "common-year Azar constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.add(months4n, options);
}, "common-year Azar rejects with 31");

TemporalHelpers.assertPlainDate(
  date13640131.add(months3n),
  1363, 10, "M10", 30, "common-year Dey constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.add(months3n, options);
}, "common-year Dey rejects with 31");

TemporalHelpers.assertPlainDate(
  date13640131.add(months2n),
  1363, 11, "M11", 30, "common-year Bahman constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.add(months2n, options);
}, "common-year Bahman rejects with 31");

TemporalHelpers.assertPlainDate(
  date13640131.add(months1n),
  1363, 12, "M12", 29, "common-year Esfand constrains to 29",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.add(months1n, options);
}, "common-year Esfand rejects with 31");

// Leap year, backwards

TemporalHelpers.assertPlainDate(
  date13630131.add(months12n, options),
  1362, 1, "M01", 31, "leap-year Farvardin does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13630131.add(months11n, options),
  1362, 2, "M02", 31, "leap-year Ordibehesht does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13630131.add(months10n, options),
  1362, 3, "M03", 31, "leap-year Khordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13630131.add(months9n, options),
  1362, 4, "M04", 31, "leap-year Tir does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13630131.add(months8n, options),
  1362, 5, "M05", 31, "leap-year Mordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13630131.add(months7n, options),
  1362, 6, "M06", 31, "leap-year Shahrivar does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  date13630131.add(months6n),
  1362, 7, "M07", 30, "leap-year Mehr constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.add(months6n, options);
}, "leap-year Mehr rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months5n),
  1362, 8, "M08", 30, "leap-year Aban constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.add(months5n, options);
}, "leap-year Aban rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months4n),
  1362, 9, "M09", 30, "leap-year Azar constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.add(months4n, options);
}, "leap-year Azar rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months3n),
  1362, 10, "M10", 30, "leap-year Dey constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.add(months3n, options);
}, "leap-year Dey rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months2n),
  1362, 11, "M11", 30, "leap-year Bahman constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.add(months2n, options);
}, "leap-year Bahman rejects with 31");

TemporalHelpers.assertPlainDate(
  date13630131.add(months1n),
  1362, 12, "M12", 30, "leap-year Esfand constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.add(months1n, options);
}, "leap-year Esfand rejects with 31");
