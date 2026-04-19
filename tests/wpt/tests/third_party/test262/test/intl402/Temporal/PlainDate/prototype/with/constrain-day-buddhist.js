// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Check constraining days to the end of a month (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";
const options = { overflow: "reject" };

const common0131 = Temporal.PlainDate.from({ year: 2562, monthCode: "M01", day: 31, calendar }, options);
const leap0131 = Temporal.PlainDate.from({ year: 2559, monthCode: "M01", day: 31, calendar }, options);

// Common year

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M02" }),
  2562, 2, "M02", 28, "common-year Feb constrains to 28",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M02" }, options);
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M03" }, options),
  2562, 3, "M03", 31, "common-year Mar does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M04" }),
  2562, 4, "M04", 30, "common-year Apr constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M04" }, options);
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M05" }, options),
  2562, 5, "M05", 31, "common-year May does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M06" }),
  2562, 6, "M06", 30, "common-year Jun constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M06" }, options);
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M07" }, options),
  2562, 7, "M07", 31, "common-year Jul does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M08" }, options),
  2562, 8, "M08", 31, "common-year Aug does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M09" }),
  2562, 9, "M09", 30, "common-year Sep constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M09" }, options);
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M10" }, options),
  2562, 10, "M10", 31, "common-year Oct does not reject 31",
  "be", 2562);

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M11" }),
  2562, 11, "M11", 30, "common-year Nov constrains to 30",
  "be", 2562);
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M11" }, options);
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M12" }, options),
  2562, 12, "M12", 31, "common-year Dec does not reject 31",
  "be", 2562);

// Leap year

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M02" }),
  2559, 2, "M02", 29, "leap-year Feb constrains to 29",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M02" }, options);
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M03" }, options),
  2559, 3, "M03", 31, "leap-year Mar does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M04" }),
  2559, 4, "M04", 30, "leap-year Apr constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M04" }, options);
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M05" }, options),
  2559, 5, "M05", 31, "leap-year May does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M06" }),
  2559, 6, "M06", 30, "leap-year Jun constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M06" }, options);
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M07" }, options),
  2559, 7, "M07", 31, "leap-year Jul does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M08" }, options),
  2559, 8, "M08", 31, "leap-year Aug does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M09" }),
  2559, 9, "M09", 30, "leap-year Sep constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M09" }, options);
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M10" }, options),
  2559, 10, "M10", 31, "leap-year Oct does not reject 31",
  "be", 2559);

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M11" }),
  2559, 11, "M11", 30, "leap-year Nov constrains to 30",
  "be", 2559);
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M11" }, options);
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M12" }, options),
  2559, 12, "M12", 31, "leap-year Dec does not reject 31",
  "be", 2559);
