// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: >
  Check various basic calculations involving constraining days to the end of a
  month (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";
const options = { overflow: "reject" };

const common0131 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M01", day: 31, hour: 12, minute: 34, calendar }, options);
const common1231 = Temporal.PlainDateTime.from({ year: 2562, monthCode: "M12", day: 31, hour: 12, minute: 34, calendar }, options);
const leap0131 = Temporal.PlainDateTime.from({ year: 2559, monthCode: "M01", day: 31, hour: 12, minute: 34, calendar }, options);
const leap1231 = Temporal.PlainDateTime.from({ year: 2559, monthCode: "M12", day: 31, hour: 12, minute: 34, calendar }, options);

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

// Common year, forwards

TemporalHelpers.assertPlainDateTime(
  common0131.add(months1),
  2562, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "common-year Feb constrains to 28",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.add(months1, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.add(months2, options),
  2562, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "common-year Mar does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common0131.add(months3),
  2562, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "common-year Apr constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.add(months3, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.add(months4, options),
  2562, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "common-year May does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common0131.add(months5),
  2562, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "common-year Jun constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.add(months5, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.add(months6, options),
  2562, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "common-year Jul does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common0131.add(months7, options),
  2562, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "common-year Aug does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common0131.add(months8),
  2562, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Sep constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.add(months8, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.add(months9, options),
  2562, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "common-year Oct does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common0131.add(months10),
  2562, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Nov constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.add(months10, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.add(months11, options),
  2562, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "common-year Dec does not reject 31",
  "be", 2562);

// Leap year, forwards

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months1),
  2559, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "leap-year Feb constrains to 29",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.add(months1, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months2, options),
  2559, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "leap-year Mar does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months3),
  2559, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "leap-year Apr constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.add(months3, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months4, options),
  2559, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "leap-year May does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months5),
  2559, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "leap-year Jun constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.add(months5, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months6, options),
  2559, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "leap-year Jul does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months7, options),
  2559, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "leap-year Aug does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months8),
  2559, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Sep constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.add(months8, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months9, options),
  2559, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "leap-year Oct does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months10),
  2559, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Nov constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.add(months10, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.add(months11, options),
  2559, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "leap-year Dec does not reject 31",
  "be", 2559);

// Common year, backwards

TemporalHelpers.assertPlainDateTime(
  common1231.add(months1n),
  2562, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Nov constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common1231.add(months1n, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.add(months2n, options),
  2562, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "common-year Oct does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common1231.add(months3n),
  2562, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Sep constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common1231.add(months3n, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.add(months4n, options),
  2562, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "common-year Aug does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common1231.add(months5n, options),
  2562, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "common-year Jul does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common1231.add(months6n),
  2562, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "common-year Jun constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common1231.add(months6n, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.add(months7n, options),
  2562, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "common-year May does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common1231.add(months8n),
  2562, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "common-year Apr constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common1231.add(months8n, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.add(months9n, options),
  2562, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "common-year Mar does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDateTime(
  common1231.add(months10n),
  2562, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "common-year Feb constrains to 28",
  "be", 2562);
assert.throws(RangeError, function () {
  common1231.add(months10n, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common1231.add(months11n, options),
  2562, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "common-year Jan does not reject 31",
  "be", 2562);

// Leap year, backwards

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months1n),
  2559, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Nov constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap1231.add(months1n, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months2n, options),
  2559, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "leap-year Oct does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months3n),
  2559, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Sep constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap1231.add(months3n, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months4n, options),
  2559, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "leap-year Aug does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months5n, options),
  2559, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "leap-year Jul does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months6n),
  2559, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "leap-year Jun constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap1231.add(months6n, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months7n, options),
  2559, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "leap-year May does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months8n),
  2559, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "leap-year Apr constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap1231.add(months8n, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months9n, options),
  2559, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "leap-year Mar does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months10n),
  2559, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "leap-year Feb constrains to 29",
  "be", 2559);
assert.throws(RangeError, function () {
  leap1231.add(months10n, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap1231.add(months11n, options),
  2559, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "leap-year Jan does not reject 31",
  "be", 2559);
