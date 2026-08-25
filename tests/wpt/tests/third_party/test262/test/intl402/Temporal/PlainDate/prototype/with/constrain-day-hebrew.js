// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Constraining the day at end of month (hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

// 30-day months: 01, 05, 05L, 07, 09, 11
// 29-day months: 04, 06, 08, 10, 12
//
// Cheshvan and Kislev (02, 03) have 29 or 30 days, independent of leap years.
// Deficient - Cheshvan and Kislev have 29 days
// Regular - Cheshvan has 29 days, Kislev 30
// Complete - Cheshvan and Kislev have 30 days
//
// Some recent years of each type: 
// 5778 - regular common year
// 5779 - complete leap year
// 5781 - deficient common year
// 5782 - regular leap year
// 5783 - complete common year
// 5784 - deficient leap year

// Years

const rcM03 = Temporal.PlainDate.from({ year: 5778, monthCode: "M03", day: 30, calendar }, options);
const clM02 = Temporal.PlainDate.from({ year: 5779, monthCode: "M02", day: 30, calendar }, options);
const clM03 = Temporal.PlainDate.from({ year: 5779, monthCode: "M03", day: 30, calendar }, options);

TemporalHelpers.assertPlainDate(
  rcM03.with({ year: 5781 }),
  5781, 3, "M03", 29, "regular-year Kislev constrains to 29 in deficient year",
  "am", 5781);
assert.throws(RangeError, function () {
  rcM03.with({ year: 5781 }, options);
}, "regular-year Kislev rejects 30 in deficient year");

TemporalHelpers.assertPlainDate(
  clM02.with({ year: 5782 }),
  5782, 2, "M02", 29, "complete-year Cheshvan constrains to 29 in regular year",
  "am", 5782);
assert.throws(RangeError, function () {
  clM02.with({ year: 5782 }, options);
}, "complete-year Cheshvan rejects 30 in regular year");

TemporalHelpers.assertPlainDate(
  clM02.with({ year: 5781 }),
  5781, 2, "M02", 29, "complete-year Cheshvan constrains to 29 in deficient year",
  "am", 5781);
assert.throws(RangeError, function () {
  clM02.with({ year: 5781 }, options);
}, "complete-year Cheshvan rejects 30 in deficient year");

TemporalHelpers.assertPlainDate(
  clM03.with({ year: 5781 }),
  5781, 3, "M03", 29, "complete-year Kislev constrains to 29 in deficient year",
  "am", 5781);
assert.throws(RangeError, function () {
  clM03.with({ year: 5781 }, options);
}, "complete-year Kislev rejects 30 in deficient year");

// Months

const dc0130 = Temporal.PlainDate.from({ year: 5781, monthCode: "M01", day: 30, calendar }, options);
const rc0130 = Temporal.PlainDate.from({ year: 5778, monthCode: "M01", day: 30, calendar }, options);
const cc0130 = Temporal.PlainDate.from({ year: 5783, monthCode: "M01", day: 30, calendar }, options);
const dl0130 = Temporal.PlainDate.from({ year: 5784, monthCode: "M01", day: 30, calendar }, options);
const rl0130 = Temporal.PlainDate.from({ year: 5782, monthCode: "M01", day: 30, calendar }, options);
const cl0130 = Temporal.PlainDate.from({ year: 5779, monthCode: "M01", day: 30, calendar }, options);

// Common year

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M02" }),
  5778, 2, "M02", 29, "regular common year Cheshvan constrains to 29",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.with({ monthCode: "M02" }, options);
}, "regular common year Cheshvan rejects 30");

TemporalHelpers.assertPlainDate(
  cc0130.with({ monthCode: "M02" }, options),
  5783, 2, "M02", 30, "regular common year Cheshvan does not reject 30",
  "am", 5783);

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M03" }, options),
  5778, 3, "M03", 30, "regular common year Kislev does not reject 30",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  dc0130.with({ monthCode: "M03" }),
  5781, 3, "M03", 29, "deficient common year Kislev constrains to 29",
  "am", 5781);
assert.throws(RangeError, function () {
  dc0130.with({ monthCode: "M03" }, options);
}, "deficient common year Kislev rejects 30");

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M04" }),
  5778, 4, "M04", 29, "common-year Tevet constrains to 29",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.with({ monthCode: "M04" }, options);
}, "common-year Tevet rejects 30");

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M05" }, options),
  5778, 5, "M05", 30, "common-year Shevat does not reject 30",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M06" }),
  5778, 6, "M06", 29, "common-year Adar constrains to 29",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.with({ monthCode: "M06" }, options);
}, "common-year Adar rejects 30");

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M07" }, options),
  5778, 7, "M07", 30, "common-year Nisan does not reject 30",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M08" }),
  5778, 8, "M08", 29, "common-year Iyar constrains to 29",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.with({ monthCode: "M08" }, options);
}, "common-year Iyar rejects 30");

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M09" }, options),
  5778, 9, "M09", 30, "common-year Sivan does not reject 30",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M10" }),
  5778, 10, "M10", 29, "common-year Tammuz constrains to 29",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.with({ monthCode: "M10" }, options);
}, "common-year Tammuz rejects 30");

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M11" }, options),
  5778, 11, "M11", 30, "common-year Av does not reject 30",
  "am", 5778);

TemporalHelpers.assertPlainDate(
  rc0130.with({ monthCode: "M12" }),
  5778, 12, "M12", 29, "common-year Elul constrains to 29",
  "am", 5778);
assert.throws(RangeError, function () {
  rc0130.with({ monthCode: "M12" }, options);
}, "common-year Elul rejects 30");

// Leap year

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M02" }),
  5782, 2, "M02", 29, "regular leap year Cheshvan constrains to 29",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.with({ monthCode: "M02" }, options);
}, "regular leap year Cheshvan rejects 30");

TemporalHelpers.assertPlainDate(
  cl0130.with({ monthCode: "M02" }, options),
  5779, 2, "M02", 30, "regular leap year Cheshvan does not reject 30",
  "am", 5779);

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M03" }, options),
  5782, 3, "M03", 30, "regular leap year Kislev does not reject 30",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  dl0130.with({ monthCode: "M03" }),
  5784, 3, "M03", 29, "deficient leap year Kislev constrains to 29",
  "am", 5784);
assert.throws(RangeError, function () {
  dl0130.with({ monthCode: "M03" }, options);
}, "deficient leap year Kislev rejects 30");

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M04" }),
  5782, 4, "M04", 29, "leap-year Tevet constrains to 29",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.with({ monthCode: "M04" }, options);
}, "leap-year Tevet rejects 30");

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M05" }, options),
  5782, 5, "M05", 30, "leap-year Shevat does not reject 30",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M05L" }, options),
  5782, 6, "M05L", 30, "leap-year Adar I does not reject 30",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M06" }),
  5782, 7, "M06", 29, "leap-year Adar II constrains to 29",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.with({ monthCode: "M06" }, options);
}, "leap-year Adar II rejects 30");

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M07" }, options),
  5782, 8, "M07", 30, "leap-year Nisan does not reject 30",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M08" }),
  5782, 9, "M08", 29, "leap-year Iyar constrains to 29",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.with({ monthCode: "M08" }, options);
}, "leap-year Iyar rejects 30");

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M09" }, options),
  5782, 10, "M09", 30, "leap-year Sivan does not reject 30",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M10" }),
  5782, 11, "M10", 29, "leap-year Tammuz constrains to 29",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.with({ monthCode: "M10" }, options);
}, "leap-year Tammuz rejects 30");

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M11" }, options),
  5782, 12, "M11", 30, "leap-year Av does not reject 30",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  rl0130.with({ monthCode: "M12" }),
  5782, 13, "M12", 29, "leap-year Elul constrains to 29",
  "am", 5782);
assert.throws(RangeError, function () {
  rl0130.with({ monthCode: "M12" }, options);
}, "leap-year Elul rejects 30");
