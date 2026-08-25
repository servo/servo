// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: >
  Check various basic calculations involving constraining days to the end of a
  month (dangi calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "dangi";
const options = { overflow: "reject" };

// For convenience of the reader, a table of month lengths in the common years
// we are testing in this file:
//
// y \ m  1  2  3  4  5  6  7  8  9 10 11 12
// 2018  29 30 29 30 29 29 30 29 30 29 30 30
// 2019  30 29 30 29 30 29 29 30 29 30 29 30 
// 2020 leap year
// 2021  29 30 30 29 30 29 30 29 30 29 30 29
// 2022  30 29 30 29 30 30 29 30 29 30 29 30

// Years

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years8 = new Temporal.Duration(-8);
const years19n = new Temporal.Duration(19);

const longLeapMonth = Temporal.PlainDate.from({ year: 1979, monthCode: "M06L", day: 30, calendar }, options);
const longLeapMonth2 = Temporal.PlainDate.from({ year: 2012, monthCode: "M03L", day: 30, calendar }, options);
const date0130 = Temporal.PlainDate.from({ year: 2019, monthCode: "M01", day: 30, calendar }, options);
const date0230 = Temporal.PlainDate.from({ year: 2018, monthCode: "M02", day: 30, calendar }, options);
const date0330 = Temporal.PlainDate.from({ year: 2019, monthCode: "M03", day: 30, calendar }, options);
const date0430 = Temporal.PlainDate.from({ year: 2018, monthCode: "M04", day: 30, calendar }, options);
const date0530 = Temporal.PlainDate.from({ year: 2019, monthCode: "M05", day: 30, calendar }, options);
const date0630 = Temporal.PlainDate.from({ year: 2022, monthCode: "M06", day: 30, calendar }, options);
const date0730 = Temporal.PlainDate.from({ year: 2018, monthCode: "M07", day: 30, calendar }, options);
const date0830 = Temporal.PlainDate.from({ year: 2019, monthCode: "M08", day: 30, calendar }, options);
const date0930 = Temporal.PlainDate.from({ year: 2021, monthCode: "M09", day: 30, calendar }, options);
const date1030 = Temporal.PlainDate.from({ year: 2022, monthCode: "M10", day: 30, calendar }, options);
const date1130 = Temporal.PlainDate.from({ year: 2021, monthCode: "M11", day: 30, calendar }, options);
const date1230 = Temporal.PlainDate.from({ year: 2022, monthCode: "M12", day: 30, calendar }, options);

// Constraining leap month day 30 to day 29 of the same leap month

TemporalHelpers.assertPlainDate(
  longLeapMonth.subtract(years8),
  1987, 7, "M06L", 29, "long M06L constrains to 29 when addition lands in year with short M06L");
assert.throws(RangeError, function () {
  longLeapMonth.subtract(years8, options);
}, "long M06L rejects when addition lands in year with short M06L");

TemporalHelpers.assertPlainDate(
  longLeapMonth2.subtract(years19n),
  1993, 4, "M03L", 29, "long M03L constrains to 29 when subtraction lands in year with short M03L");
assert.throws(RangeError, function () {
  longLeapMonth2.subtract(years19n, options);
}, "long M03L rejects when subtraction lands in year with short M03L");

// Constraining day 30 of each regular month to day 29

TemporalHelpers.assertPlainDate(
  date0130.subtract(years1n),
  2018, 1, "M01", 29, "M01-30 constrains to 29");
assert.throws(RangeError, function () {
  date0130.subtract(years1n, options);
}, "M01-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0230.subtract(years1),
  2019, 2, "M02", 29, "M02-30 constrains to 29");
assert.throws(RangeError, function () {
  date0230.subtract(years1, options);
}, "M02-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0330.subtract(years1n),
  2018, 3, "M03", 29, "M03-30 constrains to 29");
assert.throws(RangeError, function () {
  date0330.subtract(years1n, options);
}, "M03-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0430.subtract(years1),
  2019, 4, "M04", 29, "M04-30 constrains to 29");
assert.throws(RangeError, function () {
  date0430.subtract(years1, options);
}, "M04-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0530.subtract(years1n),
  2018, 5, "M05", 29, "M05-30 constrains to 29");
assert.throws(RangeError, function () {
  date0530.subtract(years1n, options);
}, "M05-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0630.subtract(years1n),
  2021, 6, "M06", 29, "M06-30 constrains to 29");
assert.throws(RangeError, function () {
  date0630.subtract(years1n, options);
}, "M06-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0730.subtract(years1),
  2019, 7, "M07", 29, "M07-30 constrains to 29");
assert.throws(RangeError, function () {
  date0730.subtract(years1, options);
}, "M07-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0830.subtract(years1n),
  2018, 8, "M08", 29, "M08-30 constrains to 29");
assert.throws(RangeError, function () {
  date0830.subtract(years1n, options);
}, "M08-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0930.subtract(years1),
  2022, 9, "M09", 29, "M09-30 constrains to 29");
assert.throws(RangeError, function () {
  date0930.subtract(years1, options);
}, "M09-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date1030.subtract(years1n),
  2021, 10, "M10", 29, "M10-30 constrains to 29");
assert.throws(RangeError, function () {
  date1030.subtract(years1n, options);
}, "M10-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date1130.subtract(years1),
  2022, 11, "M11", 29, "M11-30 constrains to 29");
assert.throws(RangeError, function () {
  date1130.subtract(years1, options);
}, "M11-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date1230.subtract(years1n),
  2021, 12, "M12", 29, "M12-30 constrains to 29");
assert.throws(RangeError, function () {
  date1230.subtract(years1n, options);
}, "M12-30 rejects in year with 29");

// Months, forwards

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

TemporalHelpers.assertPlainDate(
  date0130.subtract(months1),
  2019, 2, "M02", 29, "29-day M02 constrains with addition");
assert.throws(RangeError, function () {
  date0130.subtract(months1, options);
}, "29-day M02 rejects 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months2, options),
  2019, 3, "M03", 30, "30-day M03 does not reject 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months3),
  2019, 4, "M04", 29, "29-day M04 constrains with addition");
assert.throws(RangeError, function () {
  date0130.subtract(months3, options);
}, "29-day M04 rejects 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months4, options),
  2019, 5, "M05", 30, "30-day M05 does not reject 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months5),
  2019, 6, "M06", 29, "29-day M06 constrains with addition");
assert.throws(RangeError, function () {
  date0130.subtract(months5, options);
}, "29-day M06 rejects 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months6),
  2019, 7, "M07", 29, "29-day M07 constrains with addition");
assert.throws(RangeError, function () {
  date0130.subtract(months6, options);
}, "29-day M07 rejects 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months7, options),
  2019, 8, "M08", 30, "30-day M08 does not reject 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months8),
  2019, 9, "M09", 29, "29-day M09 constrains with addition");
assert.throws(RangeError, function () {
  date0130.subtract(months8, options);
}, "29-day M09 rejects 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months9, options),
  2019, 10, "M10", 30, "30-day M10 does not reject 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months10),
  2019, 11, "M11", 29, "29-day M11 constrains with addition");
assert.throws(RangeError, function () {
  date0130.subtract(months10, options);
}, "29-day M12 rejects 30 with addition");

TemporalHelpers.assertPlainDate(
  date0130.subtract(months11, options),
  2019, 12, "M12", 30, "30-day M12 does not reject 30 with addition");

// Months, backwards

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

TemporalHelpers.assertPlainDate(
  date1230.subtract(months1n),
  2022, 11, "M11", 29, "29-day M11 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.subtract(months1n, options);
}, "29-day M11 rejects 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months2n, options),
  2022, 10, "M10", 30, "30-day M10 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months3n),
  2022, 9, "M09", 29, "29-day M09 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.subtract(months3n, options);
}, "29-day M09 rejects 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months4n, options),
  2022, 8, "M08", 30, "30-day M08 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months5n),
  2022, 7, "M07", 29, "29-day M07 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.subtract(months5n, options);
}, "29-day M07 rejects 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months6n, options),
  2022, 6, "M06", 30, "30-day M06 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months7n, options),
  2022, 5, "M05", 30, "30-day M05 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months8n),
  2022, 4, "M04", 29, "29-day M04 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.subtract(months8n, options);
}, "29-day M04 rejects 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months9n, options),
  2022, 3, "M03", 30, "30-day M03 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months10n),
  2022, 2, "M02", 29, "29-day M02 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.subtract(months10n, options);
}, "29-day M02 rejects 30 with subtraction");

TemporalHelpers.assertPlainDate(
  date1230.subtract(months11n, options),
  2022, 1, "M01", 30, "30-day M01 does not reject 30 with subtraction");
