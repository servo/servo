// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Check constraining days to the end of a month
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const common0131 = new Temporal.PlainDate(2019, 1, 31);
const leap0131 = new Temporal.PlainDate(2016, 1, 31);

// Common year

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M02" }),
  2019, 2, "M02", 28, "common-year Feb constrains to 28");
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M02" }, { overflow: "reject" });
}, "common-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M03" }, { overflow: "reject" }),
  2019, 3, "M03", 31, "common-year Mar does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M04" }),
  2019, 4, "M04", 30, "common-year Apr constrains to 30");
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M04" }, { overflow: "reject" });
}, "common-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M05" }, { overflow: "reject" }),
  2019, 5, "M05", 31, "common-year May does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M06" }),
  2019, 6, "M06", 30, "common-year Jun constrains to 30");
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M06" }, { overflow: "reject" });
}, "common-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M07" }, { overflow: "reject" }),
  2019, 7, "M07", 31, "common-year Jul does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M08" }, { overflow: "reject" }),
  2019, 8, "M08", 31, "common-year Aug does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M09" }),
  2019, 9, "M09", 30, "common-year Sep constrains to 30");
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M09" }, { overflow: "reject" });
}, "common-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M10" }, { overflow: "reject" }),
  2019, 10, "M10", 31, "common-year Oct does not reject 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M11" }),
  2019, 11, "M11", 30, "common-year Nov constrains to 30");
assert.throws(RangeError, function () {
  common0131.with({ monthCode: "M11" }, { overflow: "reject" });
}, "common-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  common0131.with({ monthCode: "M12" }, { overflow: "reject" }),
  2019, 12, "M12", 31, "common-year Dec does not reject 31");

// Leap year, forwards

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M02" }),
  2016, 2, "M02", 29, "leap-year Feb constrains to 29");
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M02" }, { overflow: "reject" });
}, "leap-year Feb rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M03" }, { overflow: "reject" }),
  2016, 3, "M03", 31, "leap-year Mar does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M04" }),
  2016, 4, "M04", 30, "leap-year Apr constrains to 30");
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M04" }, { overflow: "reject" });
}, "leap-year Apr rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M05" }, { overflow: "reject" }),
  2016, 5, "M05", 31, "leap-year May does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M06" }),
  2016, 6, "M06", 30, "leap-year Jun constrains to 30");
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M06" }, { overflow: "reject" });
}, "leap-year Jun rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M07" }, { overflow: "reject" }),
  2016, 7, "M07", 31, "leap-year Jul does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M08" }, { overflow: "reject" }),
  2016, 8, "M08", 31, "leap-year Aug does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M09" }),
  2016, 9, "M09", 30, "leap-year Sep constrains to 30");
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M09" }, { overflow: "reject" });
}, "leap-year Sep rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M10" }, { overflow: "reject" }),
  2016, 10, "M10", 31, "leap-year Oct does not reject 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M11" }),
  2016, 11, "M11", 30, "leap-year Nov constrains to 30");
assert.throws(RangeError, function () {
  leap0131.with({ monthCode: "M11" }, { overflow: "reject" });
}, "leap-year Nov rejects with 31");

TemporalHelpers.assertPlainDate(
  leap0131.with({ monthCode: "M12" }, { overflow: "reject" }),
  2016, 12, "M12", 31, "leap-year Dec does not reject 31");
