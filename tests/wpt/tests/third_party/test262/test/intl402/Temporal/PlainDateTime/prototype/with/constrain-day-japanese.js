// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Check constraining days to the end of a month (japanese calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

const common0131 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M01", day: 31, hour: 12, minute: 34, calendar }, options);
const leap0131 = Temporal.PlainDateTime.from({ year: 2016, monthCode: "M01", day: 31, hour: 12, minute: 34, calendar }, options);

// Common year

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M02" }),
  2019, 2, "M02", 28, 12, 34, 0, 0, 0, 0, "common-year Feb constrains to 28",
  "heisei", 31);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M02" }, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M03" }, options),
  2019, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "common-year Mar does not reject 31",
  "heisei", 31);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M04" }),
  2019, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "common-year Apr constrains to 30",
  "heisei", 31);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M04" }, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M05" }, options),
  2019, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "common-year May does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M06" }),
  2019, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "common-year Jun constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M06" }, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M07" }, options),
  2019, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "common-year Jul does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M08" }, options),
  2019, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "common-year Aug does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M09" }),
  2019, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "common-year Sep constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M09" }, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M10" }, options),
  2019, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "common-year Oct does not reject 31",
  "reiwa", 1);

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M11" }),
  2019, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "common-year Nov constrains to 30",
  "reiwa", 1);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M11" }, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  common0131.with({ monthCode: "M12" }, options),
  2019, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "common-year Dec does not reject 31",
  "reiwa", 1);

// Leap year

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M02" }),
  2016, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "leap-year Feb constrains to 29",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M02" }, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M03" }, options),
  2016, 3, "M03", 31, 12, 34, 0, 0, 0, 0, "leap-year Mar does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M04" }),
  2016, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "leap-year Apr constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M04" }, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M05" }, options),
  2016, 5, "M05", 31, 12, 34, 0, 0, 0, 0, "leap-year May does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M06" }),
  2016, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "leap-year Jun constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M06" }, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M07" }, options),
  2016, 7, "M07", 31, 12, 34, 0, 0, 0, 0, "leap-year Jul does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M08" }, options),
  2016, 8, "M08", 31, 12, 34, 0, 0, 0, 0, "leap-year Aug does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M09" }),
  2016, 9, "M09", 30, 12, 34, 0, 0, 0, 0, "leap-year Sep constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M09" }, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M10" }, options),
  2016, 10, "M10", 31, 12, 34, 0, 0, 0, 0, "leap-year Oct does not reject 31",
  "heisei", 28);

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M11" }),
  2016, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "leap-year Nov constrains to 30",
  "heisei", 28);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M11" }, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDateTime(
  leap0131.with({ monthCode: "M12" }, options),
  2016, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "leap-year Dec does not reject 31",
  "heisei", 28);
