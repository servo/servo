// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: >
  Check various basic calculations involving constraining days to the end of a month
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const common0131 = new Temporal.PlainDate(2019, 1, 31);
const common1231 = new Temporal.PlainDate(2019, 12, 31);
const leap0131 = new Temporal.PlainDate(2016, 1, 31);
const leap1231 = new Temporal.PlainDate(2016, 12, 31);

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

TemporalHelpers.assertPlainDate(
  common0131.add(months1),
  2019, 2, "M02", 28, "common-year Feb constrains to 28");
assert.throws(RangeError, function () {
  common0131.add(months1, { overflow: "reject" });
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months2, { overflow: "reject" }),
  2019, 3, "M03", 31, "common-year Mar does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months3),
  2019, 4, "M04", 30, "common-year Apr constrains to 30");
assert.throws(RangeError, function () {
  common0131.add(months3, { overflow: "reject" });
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months4, { overflow: "reject" }),
  2019, 5, "M05", 31, "common-year May does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months5),
  2019, 6, "M06", 30, "common-year Jun constrains to 30");
assert.throws(RangeError, function () {
  common0131.add(months5, { overflow: "reject" });
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months6, { overflow: "reject" }),
  2019, 7, "M07", 31, "common-year Jul does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months7, { overflow: "reject" }),
  2019, 8, "M08", 31, "common-year Aug does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months8),
  2019, 9, "M09", 30, "common-year Sep constrains to 30");
assert.throws(RangeError, function () {
  common0131.add(months8, { overflow: "reject" });
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months9, { overflow: "reject" }),
  2019, 10, "M10", 31, "common-year Oct does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months10),
  2019, 11, "M11", 30, "common-year Nov constrains to 30");
assert.throws(RangeError, function () {
  common0131.add(months10, { overflow: "reject" });
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.add(months11, { overflow: "reject" }),
  2019, 12, "M12", 31, "common-year Dec does not reject 31");

// Leap year, forwards

TemporalHelpers.assertPlainDate(
  leap0131.add(months1),
  2016, 2, "M02", 29, "leap-year Feb constrains to 29");
assert.throws(RangeError, function () {
  leap0131.add(months1, { overflow: "reject" });
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months2, { overflow: "reject" }),
  2016, 3, "M03", 31, "leap-year Mar does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months3),
  2016, 4, "M04", 30, "leap-year Apr constrains to 30");
assert.throws(RangeError, function () {
  leap0131.add(months3, { overflow: "reject" });
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months4, { overflow: "reject" }),
  2016, 5, "M05", 31, "leap-year May does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months5),
  2016, 6, "M06", 30, "leap-year Jun constrains to 30");
assert.throws(RangeError, function () {
  leap0131.add(months5, { overflow: "reject" });
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months6, { overflow: "reject" }),
  2016, 7, "M07", 31, "leap-year Jul does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months7, { overflow: "reject" }),
  2016, 8, "M08", 31, "leap-year Aug does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months8),
  2016, 9, "M09", 30, "leap-year Sep constrains to 30");
assert.throws(RangeError, function () {
  leap0131.add(months8, { overflow: "reject" });
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months9, { overflow: "reject" }),
  2016, 10, "M10", 31, "leap-year Oct does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months10),
  2016, 11, "M11", 30, "leap-year Nov constrains to 30");
assert.throws(RangeError, function () {
  leap0131.add(months10, { overflow: "reject" });
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.add(months11, { overflow: "reject" }),
  2016, 12, "M12", 31, "leap-year Dec does not reject 31");

// Common year, backwards

TemporalHelpers.assertPlainDate(
  common1231.add(months1n),
  2019, 11, "M11", 30, "common-year Nov constrains to 30");
assert.throws(RangeError, function () {
  common1231.add(months1n, { overflow: "reject" });
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months2n, { overflow: "reject" }),
  2019, 10, "M10", 31, "common-year Oct does not reject 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months3n),
  2019, 9, "M09", 30, "common-year Sep constrains to 30");
assert.throws(RangeError, function () {
  common1231.add(months3n, { overflow: "reject" });
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months4n, { overflow: "reject" }),
  2019, 8, "M08", 31, "common-year Aug does not reject 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months5n, { overflow: "reject" }),
  2019, 7, "M07", 31, "common-year Jul does not reject 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months6n),
  2019, 6, "M06", 30, "common-year Jun constrains to 30");
assert.throws(RangeError, function () {
  common1231.add(months6n, { overflow: "reject" });
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months7n, { overflow: "reject" }),
  2019, 5, "M05", 31, "common-year May does not reject 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months8n),
  2019, 4, "M04", 30, "common-year Apr constrains to 30");
assert.throws(RangeError, function () {
  common1231.add(months8n, { overflow: "reject" });
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months9n, { overflow: "reject" }),
  2019, 3, "M03", 31, "common-year Mar does not reject 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months10n),
  2019, 2, "M02", 28, "common-year Feb constrains to 28");
assert.throws(RangeError, function () {
  common1231.add(months10n, { overflow: "reject" });
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  common1231.add(months11n, { overflow: "reject" }),
  2019, 1, "M01", 31, "common-year Jan does not reject 31");

// Leap year, backwards

TemporalHelpers.assertPlainDate(
  leap1231.add(months1n),
  2016, 11, "M11", 30, "leap-year Nov constrains to 30");
assert.throws(RangeError, function () {
  leap1231.add(months1n, { overflow: "reject" });
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months2n, { overflow: "reject" }),
  2016, 10, "M10", 31, "leap-year Oct does not reject 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months3n),
  2016, 9, "M09", 30, "leap-year Sep constrains to 30");
assert.throws(RangeError, function () {
  leap1231.add(months3n, { overflow: "reject" });
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months4n, { overflow: "reject" }),
  2016, 8, "M08", 31, "leap-year Aug does not reject 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months5n, { overflow: "reject" }),
  2016, 7, "M07", 31, "leap-year Jul does not reject 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months6n),
  2016, 6, "M06", 30, "leap-year Jun constrains to 30");
assert.throws(RangeError, function () {
  leap1231.add(months6n, { overflow: "reject" });
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months7n, { overflow: "reject" }),
  2016, 5, "M05", 31, "leap-year May does not reject 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months8n),
  2016, 4, "M04", 30, "leap-year Apr constrains to 30");
assert.throws(RangeError, function () {
  leap1231.add(months8n, { overflow: "reject" });
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months9n, { overflow: "reject" }),
  2016, 3, "M03", 31, "leap-year Mar does not reject 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months10n),
  2016, 2, "M02", 29, "leap-year Feb constrains to 29");
assert.throws(RangeError, function () {
  leap1231.add(months10n, { overflow: "reject" });
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  leap1231.add(months11n, { overflow: "reject" }),
  2016, 1, "M01", 31, "leap-year Jan does not reject 31");
