// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Check constraining days to the end of a month (roc calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

const common0131 = Temporal.ZonedDateTime.from({ year: 108, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const leap0131 = Temporal.ZonedDateTime.from({ year: 105, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

// Common year

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M02" }).toPlainDateTime(),
  108, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "common-year Feb constrains to 28",
  "roc", 108);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M02" }, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M03" }, options).toPlainDateTime(),
  108, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "common-year Mar does not reject 31",
  "roc", 108);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M04" }).toPlainDateTime(),
  108, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "common-year Apr constrains to 30",
  "roc", 108);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M04" }, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M05" }, options).toPlainDateTime(),
  108, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "common-year May does not reject 31",
  "roc", 108);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M06" }).toPlainDateTime(),
  108, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "common-year Jun constrains to 30",
  "roc", 108);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M06" }, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M07" }, options).toPlainDateTime(),
  108, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "common-year Jul does not reject 31",
  "roc", 108);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M08" }, options).toPlainDateTime(),
  108, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "common-year Aug does not reject 31",
  "roc", 108);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M09" }).toPlainDateTime(),
  108, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Sep constrains to 30",
  "roc", 108);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M09" }, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M10" }, options).toPlainDateTime(),
  108, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "common-year Oct does not reject 31",
  "roc", 108);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M11" }).toPlainDateTime(),
  108, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Nov constrains to 30",
  "roc", 108);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M11" }, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M12" }, options).toPlainDateTime(),
  108, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "common-year Dec does not reject 31",
  "roc", 108);

// Leap year

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M02" }).toPlainDateTime(),
  105, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "leap-year Feb constrains to 29",
  "roc", 105);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M02" }, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M03" }, options).toPlainDateTime(),
  105, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "leap-year Mar does not reject 31",
  "roc", 105);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M04" }).toPlainDateTime(),
  105, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "leap-year Apr constrains to 30",
  "roc", 105);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M04" }, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M05" }, options).toPlainDateTime(),
  105, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "leap-year May does not reject 31",
  "roc", 105);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M06" }).toPlainDateTime(),
  105, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "leap-year Jun constrains to 30",
  "roc", 105);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M06" }, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M07" }, options).toPlainDateTime(),
  105, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "leap-year Jul does not reject 31",
  "roc", 105);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M08" }, options).toPlainDateTime(),
  105, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "leap-year Aug does not reject 31",
  "roc", 105);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M09" }).toPlainDateTime(),
  105, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Sep constrains to 30",
  "roc", 105);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M09" }, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M10" }, options).toPlainDateTime(),
  105, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "leap-year Oct does not reject 31",
  "roc", 105);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M11" }).toPlainDateTime(),
  105, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Nov constrains to 30",
  "roc", 105);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M11" }, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M12" }, options).toPlainDateTime(),
  105, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "leap-year Dec does not reject 31",
  "roc", 105);
