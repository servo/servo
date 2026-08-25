// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: >
  Check various basic calculations involving constraining days to the end of a
  month (japanese calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

const common0131 = Temporal.PlainDate.from({ year: 2019, monthCode: "M01", day: 31, calendar }, options);
const common1231 = Temporal.PlainDate.from({ year: 2019, monthCode: "M12", day: 31, calendar }, options);
const leap0131 = Temporal.PlainDate.from({ year: 2016, monthCode: "M01", day: 31, calendar }, options);
const leap1231 = Temporal.PlainDate.from({ year: 2016, monthCode: "M12", day: 31, calendar }, options);

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

TemporalHelpers.assertPlainDate(
  common0131.subtract(months1),
  2019, 2, "M02", 28, "common-year Feb constrains to 28",
  "heisei", 31);
assert.throws(RangeError, function () {
  common0131.subtract(months1, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.subtract(months2, options),
  2019, 3, "M03", 31, "common-year Mar does not reject 31",
  "heisei", 31);

TemporalHelpers.assertPlainDate(
  common0131.subtract(months3),
  2019, 4, "M04", 30, "common-year Apr constrains to 30",
  "heisei", 31);
assert.throws(RangeError, function () {
  common0131.subtract(months3, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.subtract(months4, options),
  2019, 5, "M05", 31, "common-year May does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDate(
  common0131.subtract(months5),
  2019, 6, "M06", 30, "common-year Jun constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common0131.subtract(months5, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.subtract(months6, options),
  2019, 7, "M07", 31, "common-year Jul does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDate(
  common0131.subtract(months7, options),
  2019, 8, "M08", 31, "common-year Aug does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDate(
  common0131.subtract(months8),
  2019, 9, "M09", 30, "common-year Sep constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common0131.subtract(months8, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.subtract(months9, options),
  2019, 10, "M10", 31, "common-year Oct does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDate(
  common0131.subtract(months10),
  2019, 11, "M11", 30, "common-year Nov constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common0131.subtract(months10, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.subtract(months11, options),
  2019, 12, "M12", 31, "common-year Dec does not reject 31",
  "reiwa", 1);

// Leap year, forwards

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months1),
  2016, 2, "M02", 29, "leap-year Feb constrains to 29",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.subtract(months1, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months2, options),
  2016, 3, "M03", 31, "leap-year Mar does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months3),
  2016, 4, "M04", 30, "leap-year Apr constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.subtract(months3, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months4, options),
  2016, 5, "M05", 31, "leap-year May does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months5),
  2016, 6, "M06", 30, "leap-year Jun constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.subtract(months5, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months6, options),
  2016, 7, "M07", 31, "leap-year Jul does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months7, options),
  2016, 8, "M08", 31, "leap-year Aug does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months8),
  2016, 9, "M09", 30, "leap-year Sep constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.subtract(months8, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months9, options),
  2016, 10, "M10", 31, "leap-year Oct does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months10),
  2016, 11, "M11", 30, "leap-year Nov constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.subtract(months10, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.subtract(months11, options),
  2016, 12, "M12", 31, "leap-year Dec does not reject 31",
  "heisei", 28);

// Common year, backwards

TemporalHelpers.assertPlainDate(
  common1231.subtract(months1n),
  2019, 11, "M11", 30, "common-year Nov constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common1231.subtract(months1n, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.subtract(months2n, options),
  2019, 10, "M10", 31, "common-year Oct does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDate(
  common1231.subtract(months3n),
  2019, 9, "M09", 30, "common-year Sep constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common1231.subtract(months3n, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.subtract(months4n, options),
  2019, 8, "M08", 31, "common-year Aug does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDate(
  common1231.subtract(months5n, options),
  2019, 7, "M07", 31, "common-year Jul does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDate(
  common1231.subtract(months6n),
  2019, 6, "M06", 30, "common-year Jun constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common1231.subtract(months6n, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.subtract(months7n, options),
  2019, 5, "M05", 31, "common-year May does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDate(
  common1231.subtract(months8n),
  2019, 4, "M04", 30, "common-year Apr constrains to 30",
  "heisei", 31);
assert.throws(RangeError, function () {
  common1231.subtract(months8n, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.subtract(months9n, options),
  2019, 3, "M03", 31, "common-year Mar does not reject 31",
  "heisei", 31);

TemporalHelpers.assertPlainDate(
  common1231.subtract(months10n),
  2019, 2, "M02", 28, "common-year Feb constrains to 28",
  "heisei", 31);
assert.throws(RangeError, function () {
  common1231.subtract(months10n, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.subtract(months11n, options),
  2019, 1, "M01", 31, "common-year Jan does not reject 31",
  "heisei", 31);

// Leap year, backwards

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months1n),
  2016, 11, "M11", 30, "leap-year Nov constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap1231.subtract(months1n, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months2n, options),
  2016, 10, "M10", 31, "leap-year Oct does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months3n),
  2016, 9, "M09", 30, "leap-year Sep constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap1231.subtract(months3n, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months4n, options),
  2016, 8, "M08", 31, "leap-year Aug does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months5n, options),
  2016, 7, "M07", 31, "leap-year Jul does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months6n),
  2016, 6, "M06", 30, "leap-year Jun constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap1231.subtract(months6n, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months7n, options),
  2016, 5, "M05", 31, "leap-year May does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months8n),
  2016, 4, "M04", 30, "leap-year Apr constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap1231.subtract(months8n, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months9n, options),
  2016, 3, "M03", 31, "leap-year Mar does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months10n),
  2016, 2, "M02", 29, "leap-year Feb constrains to 29",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap1231.subtract(months10n, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.subtract(months11n, options),
  2016, 1, "M01", 31, "leap-year Jan does not reject 31",
  "heisei", 28);
