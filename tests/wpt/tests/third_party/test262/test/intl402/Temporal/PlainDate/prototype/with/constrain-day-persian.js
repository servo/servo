// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Constraining the day for 29/30-day months in persian calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "persian";
const options = { overflow: "reject" };

// 31-day months: 01, 02, 03, 04, 05, 06
// 30-day months: 07, 08, 09, 10, 11
// Month 12 (Esfand) has 29 days in common years and 30 in leap years.
// 1362 is a leap year, 1363 and 1364 are common years.

const common0131 = Temporal.PlainDate.from({ year: 1363, monthCode: "M01", day: 31, calendar }, options);
const leap0131 = Temporal.PlainDate.from({ year: 1362, monthCode: "M01", day: 31, calendar }, options);

// Common year

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M02" }, options),
  1363, 2, "M02", 31, "common-year Ordibehesht does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M03" }, options),
  1363, 3, "M03", 31, "common-year Khordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M04" }, options),
  1363, 4, "M04", 31, "common-year Tir does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M05" }, options),
  1363, 5, "M05", 31, "common-year Mordad does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M06" }, options),
  1363, 6, "M06", 31, "common-year Shahrivar does not reject 31",
  "ap", 1363);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M07" }),
  1363, 7, "M07", 30, "common-year Mehr constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M07" }, options);
}, "common-year Mehr rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M08" }),
  1363, 8, "M08", 30, "common-year Aban constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M08" }, options);
}, "common-year Aban rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M09" }),
  1363, 9, "M09", 30, "common-year Azar constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M09" }, options);
}, "common-year Azar rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M10" }),
  1363, 10, "M10", 30, "common-year Dey constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M10" }, options);
}, "common-year Dey rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M11" }),
  1363, 11, "M11", 30, "common-year Bahman constrains to 30",
  "ap", 1363);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M11" }, options);
}, "common-year Bahman rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M12" }),
  1363, 12, "M12", 29, "common-year Esfand constrains to 29",
  "ap", 1363);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M12" }, options);
}, "common-year Esfand rejects with 31");

// Leap year

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M02" }, options),
  1362, 2, "M02", 31, "leap-year Ordibehesht does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M03" }, options),
  1362, 3, "M03", 31, "leap-year Khordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M04" }, options),
  1362, 4, "M04", 31, "leap-year Tir does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M05" }, options),
  1362, 5, "M05", 31, "leap-year Mordad does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M06" }, options),
  1362, 6, "M06", 31, "leap-year Shahrivar does not reject 31",
  "ap", 1362);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M07" }),
  1362, 7, "M07", 30, "leap-year Mehr constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M07" }, options);
}, "leap-year Mehr rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M08" }),
  1362, 8, "M08", 30, "leap-year Aban constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M08" }, options);
}, "leap-year Aban rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M09" }),
  1362, 9, "M09", 30, "leap-year Azar constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M09" }, options);
}, "leap-year Azar rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M10" }),
  1362, 10, "M10", 30, "leap-year Dey constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M10" }, options);
}, "leap-year Dey rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M11" }),
  1362, 11, "M11", 30, "leap-year Bahman constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M11" }, options);
}, "leap-year Bahman rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M12" }),
  1362, 12, "M12", 30, "leap-year Esfand constrains to 30",
  "ap", 1362);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M12" }, options);
}, "leap-year Esfand rejects with 31");
