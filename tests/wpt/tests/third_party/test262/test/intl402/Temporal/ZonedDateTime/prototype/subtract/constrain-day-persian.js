// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
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

const date13620131 = Temporal.ZonedDateTime.from({ year: 1362, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13630131 = Temporal.ZonedDateTime.from({ year: 1363, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date13640131 = Temporal.ZonedDateTime.from({ year: 1364, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

const months1 = new Temporal.Duration(0, -1);
const months2 = new Temporal.Duration(0, -2);
const months3 = new Temporal.Duration(0, -3);
const months4 = new Temporal.Duration(0, -4);
const months5 = new Temporal.Duration(0, -5);
const months6 = new Temporal.Duration(0, -6);
const months7 = new Temporal.Duration(0, -7);
const months8 = new Temporal.Duration(0, -8);
const months9 = new Temporal.Duration(0, -9);
const months10 = new Temporal.Duration(0, -10);
const months11 = new Temporal.Duration(0, -11);
const months1n = new Temporal.Duration(0, 1);
const months2n = new Temporal.Duration(0, 2);
const months3n = new Temporal.Duration(0, 3);
const months4n = new Temporal.Duration(0, 4);
const months5n = new Temporal.Duration(0, 5);
const months6n = new Temporal.Duration(0, 6);
const months7n = new Temporal.Duration(0, 7);
const months8n = new Temporal.Duration(0, 8);
const months9n = new Temporal.Duration(0, 9);
const months10n = new Temporal.Duration(0, 10);
const months11n = new Temporal.Duration(0, 11);
const months12n = new Temporal.Duration(0, 12);

// Common year, forwards

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months6).toPlainDateTime(),
  1363, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "common-year Mehr constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.subtract(months6, options);
}, "common-year Mehr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months1, options).toPlainDateTime(),
  1363, 2, "M02", 31, 12, 34, 0, 0, 0, 0, "common-year Ordibehesht does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months7).toPlainDateTime(),
  1363, 8, "M08", 30, 12, 34, 0, 0, 0, 0, "common-year Aban constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.subtract(months7, options);
}, "common-year Aban rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months2, options).toPlainDateTime(),
  1363, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "common-year Khordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months8).toPlainDateTime(),
  1363, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Azar constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13620131.subtract(months8, options);
}, "common-year Azar rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months3, options).toPlainDateTime(),
  1363, 4, "M04", 31, 12, 34, 0, 0, 0, 0, "common-year Tir does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months9).toPlainDateTime(),
  1363, 10, "M10", 30, 12, 34, 0, 0, 0, 0, "common-year Dey constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.subtract(months9, options);
}, "common-year Dey rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months4, options).toPlainDateTime(),
  1363, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "common-year Mordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months10).toPlainDateTime(),
  1363, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Bahman constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.subtract(months10, options);
}, "common-year Bahman rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months5, options).toPlainDateTime(),
  1363, 6, "M06", 31, 12, 34, 0, 0, 0, 0, "common-year Shahrivar does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months11).toPlainDateTime(),
  1363, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "common-year Esfand constrains to 29",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13630131.subtract(months11, options);
}, "common-year Esfand rejects with 31");

// Leap year, forwards

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months6).toPlainDateTime(),
  1362, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "leap-year Mehr constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.subtract(months6, options);
}, "leap-year Mehr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months1, options).toPlainDateTime(),
  1362, 2, "M02", 31, 12, 34, 0, 0, 0, 0, "leap-year Ordibehesht does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months7).toPlainDateTime(),
  1362, 8, "M08", 30, 12, 34, 0, 0, 0, 0, "leap-year Aban constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.subtract(months7, options);
}, "leap-year Aban rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months2, options).toPlainDateTime(),
  1362, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "leap-year Khordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months8).toPlainDateTime(),
  1362, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Azar constrains to 29",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.subtract(months8, options);
}, "leap-year Azar rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months3, options).toPlainDateTime(),
  1362, 4, "M04", 31, 12, 34, 0, 0, 0, 0, "leap-year Tir does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months9).toPlainDateTime(),
  1362, 10, "M10", 30, 12, 34, 0, 0, 0, 0, "leap-year Dey constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.subtract(months9, options);
}, "leap-year Dey rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months4, options).toPlainDateTime(),
  1362, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "leap-year Mordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months10).toPlainDateTime(),
  1362, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Bahman constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13620131.subtract(months10, options);
}, "leap-year Bahman rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months5, options).toPlainDateTime(),
  1362, 6, "M06", 31, 12, 34, 0, 0, 0, 0, "leap-year Shahrivar does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13620131.subtract(months11).toPlainDateTime(),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "leap-year Esfand constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.subtract(months11, options);
}, "leap-year Esfand rejects with 30");

// Common year, backwards

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months12n, options).toPlainDateTime(),
  1363, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "common-year Farvardin does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months11n, options).toPlainDateTime(),
  1363, 2, "M02", 31, 12, 34, 0, 0, 0, 0, "common-year Ordibehesht does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months10n, options).toPlainDateTime(),
  1363, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "common-year Khordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months9n, options).toPlainDateTime(),
  1363, 4, "M04", 31, 12, 34, 0, 0, 0, 0, "common-year Tir does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months8n, options).toPlainDateTime(),
  1363, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "common-year Mordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months7n, options).toPlainDateTime(),
  1363, 6, "M06", 31, 12, 34, 0, 0, 0, 0, "common-year Shahrivar does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months6n).toPlainDateTime(),
  1363, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "common-year Mehr constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.subtract(months6n, options);
}, "common-year Mehr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months5n).toPlainDateTime(),
  1363, 8, "M08", 30, 12, 34, 0, 0, 0, 0, "common-year Aban constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.subtract(months5n, options);
}, "common-year Aban rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months4n).toPlainDateTime(),
  1363, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Azar constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.subtract(months4n, options);
}, "common-year Azar rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months3n).toPlainDateTime(),
  1363, 10, "M10", 30, 12, 34, 0, 0, 0, 0, "common-year Dey constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.subtract(months3n, options);
}, "common-year Dey rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months2n).toPlainDateTime(),
  1363, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Bahman constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.subtract(months2n, options);
}, "common-year Bahman rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13640131.subtract(months1n).toPlainDateTime(),
  1363, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "common-year Esfand constrains to 29",
  "ap", 1363);
assert.throws(RangeError, function () {
  date13640131.subtract(months1n, options);
}, "common-year Esfand rejects with 31");

// Leap year, backwards

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months12n, options).toPlainDateTime(),
  1362, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "leap-year Farvardin does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months11n, options).toPlainDateTime(),
  1362, 2, "M02", 31, 12, 34, 0, 0, 0, 0, "leap-year Ordibehesht does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months10n, options).toPlainDateTime(),
  1362, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "leap-year Khordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months9n, options).toPlainDateTime(),
  1362, 4, "M04", 31, 12, 34, 0, 0, 0, 0, "leap-year Tir does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months8n, options).toPlainDateTime(),
  1362, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "leap-year Mordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months7n, options).toPlainDateTime(),
  1362, 6, "M06", 31, 12, 34, 0, 0, 0, 0, "leap-year Shahrivar does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months6n).toPlainDateTime(),
  1362, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "leap-year Mehr constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.subtract(months6n, options);
}, "leap-year Mehr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months5n).toPlainDateTime(),
  1362, 8, "M08", 30, 12, 34, 0, 0, 0, 0, "leap-year Aban constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.subtract(months5n, options);
}, "leap-year Aban rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months4n).toPlainDateTime(),
  1362, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Azar constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.subtract(months4n, options);
}, "leap-year Azar rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months3n).toPlainDateTime(),
  1362, 10, "M10", 30, 12, 34, 0, 0, 0, 0, "leap-year Dey constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.subtract(months3n, options);
}, "leap-year Dey rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months2n).toPlainDateTime(),
  1362, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Bahman constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.subtract(months2n, options);
}, "leap-year Bahman rejects with 31");

TemporalHelpers.assertPlainDateTime(
  date13630131.subtract(months1n).toPlainDateTime(),
  1362, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "leap-year Esfand constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  date13630131.subtract(months1n, options);
}, "leap-year Esfand rejects with 31");
