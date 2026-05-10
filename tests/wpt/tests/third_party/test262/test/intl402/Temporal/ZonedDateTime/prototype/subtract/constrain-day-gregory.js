// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: >
  Check various basic calculations involving constraining days to the end of a
  month (gregory calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "gregory";
const options = { overflow: "reject" };

const common0131 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const common1231 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const leap0131 = Temporal.ZonedDateTime.from({ year: 2016, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const leap1231 = Temporal.ZonedDateTime.from({ year: 2016, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

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

// Common year, forwards

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months1).toPlainDateTime(),
  2019, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "common-year Feb constrains to 28",
  "ce", 2019);
assert.throws(RangeError, function () {
  common0131.subtract(months1, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months2, options).toPlainDateTime(),
  2019, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "common-year Mar does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months3).toPlainDateTime(),
  2019, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "common-year Apr constrains to 30",
  "ce", 2019);
assert.throws(RangeError, function () {
  common0131.subtract(months3, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months4, options).toPlainDateTime(),
  2019, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "common-year May does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months5).toPlainDateTime(),
  2019, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "common-year Jun constrains to 30",
  "ce", 2019);
assert.throws(RangeError, function () {
  common0131.subtract(months5, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months6, options).toPlainDateTime(),
  2019, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "common-year Jul does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months7, options).toPlainDateTime(),
  2019, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "common-year Aug does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months8).toPlainDateTime(),
  2019, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Sep constrains to 30",
  "ce", 2019);
assert.throws(RangeError, function () {
  common0131.subtract(months8, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months9, options).toPlainDateTime(),
  2019, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "common-year Oct does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months10).toPlainDateTime(),
  2019, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Nov constrains to 30",
  "ce", 2019);
assert.throws(RangeError, function () {
  common0131.subtract(months10, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.subtract(months11, options).toPlainDateTime(),
  2019, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "common-year Dec does not reject 31",
  "ce", 2019);

// Leap year, forwards

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months1).toPlainDateTime(),
  2016, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "leap-year Feb constrains to 29",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap0131.subtract(months1, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months2, options).toPlainDateTime(),
  2016, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "leap-year Mar does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months3).toPlainDateTime(),
  2016, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "leap-year Apr constrains to 30",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap0131.subtract(months3, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months4, options).toPlainDateTime(),
  2016, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "leap-year May does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months5).toPlainDateTime(),
  2016, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "leap-year Jun constrains to 30",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap0131.subtract(months5, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months6, options).toPlainDateTime(),
  2016, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "leap-year Jul does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months7, options).toPlainDateTime(),
  2016, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "leap-year Aug does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months8).toPlainDateTime(),
  2016, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Sep constrains to 30",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap0131.subtract(months8, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months9, options).toPlainDateTime(),
  2016, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "leap-year Oct does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months10).toPlainDateTime(),
  2016, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Nov constrains to 30",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap0131.subtract(months10, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.subtract(months11, options).toPlainDateTime(),
  2016, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "leap-year Dec does not reject 31",
  "ce", 2016);

// Common year, backwards

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months1n).toPlainDateTime(),
  2019, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Nov constrains to 30",
  "ce", 2019);
assert.throws(RangeError, function () {
  common1231.subtract(months1n, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months2n, options).toPlainDateTime(),
  2019, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "common-year Oct does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months3n).toPlainDateTime(),
  2019, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Sep constrains to 30",
  "ce", 2019);
assert.throws(RangeError, function () {
  common1231.subtract(months3n, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months4n, options).toPlainDateTime(),
  2019, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "common-year Aug does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months5n, options).toPlainDateTime(),
  2019, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "common-year Jul does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months6n).toPlainDateTime(),
  2019, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "common-year Jun constrains to 30",
  "ce", 2019);
assert.throws(RangeError, function () {
  common1231.subtract(months6n, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months7n, options).toPlainDateTime(),
  2019, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "common-year May does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months8n).toPlainDateTime(),
  2019, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "common-year Apr constrains to 30",
  "ce", 2019);
assert.throws(RangeError, function () {
  common1231.subtract(months8n, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months9n, options).toPlainDateTime(),
  2019, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "common-year Mar does not reject 31",
  "ce", 2019);

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months10n).toPlainDateTime(),
  2019, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "common-year Feb constrains to 28",
  "ce", 2019);
assert.throws(RangeError, function () {
  common1231.subtract(months10n, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.subtract(months11n, options).toPlainDateTime(),
  2019, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "common-year Jan does not reject 31",
  "ce", 2019);

// Leap year, backwards

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months1n).toPlainDateTime(),
  2016, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Nov constrains to 30",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap1231.subtract(months1n, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months2n, options).toPlainDateTime(),
  2016, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "leap-year Oct does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months3n).toPlainDateTime(),
  2016, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Sep constrains to 30",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap1231.subtract(months3n, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months4n, options).toPlainDateTime(),
  2016, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "leap-year Aug does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months5n, options).toPlainDateTime(),
  2016, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "leap-year Jul does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months6n).toPlainDateTime(),
  2016, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "leap-year Jun constrains to 30",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap1231.subtract(months6n, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months7n, options).toPlainDateTime(),
  2016, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "leap-year May does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months8n).toPlainDateTime(),
  2016, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "leap-year Apr constrains to 30",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap1231.subtract(months8n, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months9n, options).toPlainDateTime(),
  2016, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "leap-year Mar does not reject 31",
  "ce", 2016);

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months10n).toPlainDateTime(),
  2016, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "leap-year Feb constrains to 29",
  "ce", 2016);
assert.throws(RangeError, function () {
  leap1231.subtract(months10n, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.subtract(months11n, options).toPlainDateTime(),
  2016, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "leap-year Jan does not reject 31",
  "ce", 2016);
