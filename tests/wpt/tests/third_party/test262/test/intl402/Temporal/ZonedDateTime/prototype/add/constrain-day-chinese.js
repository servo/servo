// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: >
  Check various basic calculations involving constraining days to the end of a
  month (chinese calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "chinese";
const options = { overflow: "reject" };

// For convenience of the reader, a table of month lengths in the common years
// we are testing in this file:
//
// y \ m  1  2  3  4  5  6  7  8  9 10 11 12
// 2018  29 30 29 30 29 29 30 29 30*29 30 30
// 2019  30 29 30 29 30 29 29 30 29 29 30 30
// 2020 leap year
// 2021  29 30 30 29 30 29 30 29 30 29 30 29
// 2022  30 29 30 29 30 30 29 30 29 30 29 30
//
// *ICU4C incorrect according to HKO data

// Years

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years8 = new Temporal.Duration(8);
const years19n = new Temporal.Duration(-19);

const longLeapMonth = Temporal.ZonedDateTime.from({ year: 2017, monthCode: "M06L", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const longLeapMonth2 = Temporal.ZonedDateTime.from({ year: 1979, monthCode: "M06L", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0130 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0230 = Temporal.ZonedDateTime.from({ year: 2018, monthCode: "M02", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0330 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0430 = Temporal.ZonedDateTime.from({ year: 2018, monthCode: "M04", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0530 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M05", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0630 = Temporal.ZonedDateTime.from({ year: 2022, monthCode: "M06", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0730 = Temporal.ZonedDateTime.from({ year: 2018, monthCode: "M07", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0830 = Temporal.ZonedDateTime.from({ year: 2019, monthCode: "M08", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date0930 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M09", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date1030 = Temporal.ZonedDateTime.from({ year: 2022, monthCode: "M10", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date1130 = Temporal.ZonedDateTime.from({ year: 2021, monthCode: "M11", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const date1230 = Temporal.ZonedDateTime.from({ year: 2022, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

// Constraining leap month day 30 to day 29 of the same leap month

TemporalHelpers.assertPlainDateTime(
  longLeapMonth.add(years8).toPlainDateTime(),
  2025, 7, "M06L", 29, 12, 34, 0, 0, 0, 0, "long M06L constrains to 29 when addition lands in year with short M06L");
assert.throws(RangeError, function () {
  longLeapMonth.add(years8, options);
}, "long M06L rejects when addition lands in year with short M06L");

TemporalHelpers.assertPlainDateTime(
  longLeapMonth2.add(years19n).toPlainDateTime(),
  1960, 7, "M06L", 29, 12, 34, 0, 0, 0, 0, "long M06L constrains to 29 when subtraction lands in year with short M06L");
assert.throws(RangeError, function () {
  longLeapMonth2.add(years19n, options);
}, "long M06L rejects when subtraction lands in year with short M06L");

// Constraining day 30 of each regular month to day 29

TemporalHelpers.assertPlainDateTime(
  date0130.add(years1n).toPlainDateTime(),
  2018, 1, "M01", 29, 12, 34, 0, 0, 0, 0, "M01-30 constrains to 29");
assert.throws(RangeError, function () {
  date0130.add(years1n, options);
}, "M01-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0230.add(years1).toPlainDateTime(),
  2019, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "M02-30 constrains to 29");
assert.throws(RangeError, function () {
  date0230.add(years1, options);
}, "M02-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0330.add(years1n).toPlainDateTime(),
  2018, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "M03-30 constrains to 29");
assert.throws(RangeError, function () {
  date0330.add(years1n, options);
}, "M03-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0430.add(years1).toPlainDateTime(),
  2019, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "M04-30 constrains to 29");
assert.throws(RangeError, function () {
  date0430.add(years1, options);
}, "M04-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0530.add(years1n).toPlainDateTime(),
  2018, 5, "M05", 29, 12, 34, 0, 0, 0, 0, "M05-30 constrains to 29");
assert.throws(RangeError, function () {
  date0530.add(years1n, options);
}, "M05-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0630.add(years1n).toPlainDateTime(),
  2021, 6, "M06", 29, 12, 34, 0, 0, 0, 0, "M06-30 constrains to 29");
assert.throws(RangeError, function () {
  date0630.add(years1n, options);
}, "M06-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0730.add(years1).toPlainDateTime(),
  2019, 7, "M07", 29, 12, 34, 0, 0, 0, 0, "M07-30 constrains to 29");
assert.throws(RangeError, function () {
  date0730.add(years1, options);
}, "M07-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0830.add(years1n).toPlainDateTime(),
  2018, 8, "M08", 29, 12, 34, 0, 0, 0, 0, "M08-30 constrains to 29");
assert.throws(RangeError, function () {
  date0830.add(years1n, options);
}, "M08-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0930.add(years1).toPlainDateTime(),
  2022, 9, "M09", 29, 12, 34, 0, 0, 0, 0, "M09-30 constrains to 29");
assert.throws(RangeError, function () {
  date0930.add(years1, options);
}, "M09-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date1030.add(years1n).toPlainDateTime(),
  2021, 10, "M10", 29, 12, 34, 0, 0, 0, 0, "M10-30 constrains to 29");
assert.throws(RangeError, function () {
  date1030.add(years1n, options);
}, "M10-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date1130.add(years1).toPlainDateTime(),
  2022, 11, "M11", 29, 12, 34, 0, 0, 0, 0, "M11-30 constrains to 29");
assert.throws(RangeError, function () {
  date1130.add(years1, options);
}, "M11-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date1230.add(years1n).toPlainDateTime(),
  2021, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "M12-30 constrains to 29");
assert.throws(RangeError, function () {
  date1230.add(years1n, options);
}, "M12-30 rejects in year with 29");

// Months, forwards

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

TemporalHelpers.assertPlainDateTime(
  date0130.add(months1).toPlainDateTime(),
  2019, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "29-day M02 constrains with addition");
assert.throws(RangeError, function () {
  date0130.add(months1, options);
}, "29-day M02 rejects 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months2, options).toPlainDateTime(),
  2019, 3, "M03", 30, 12, 34, 0, 0, 0, 0, "30-day M03 does not reject 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months3).toPlainDateTime(),
  2019, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "29-day M04 constrains with addition");
assert.throws(RangeError, function () {
  date0130.add(months3, options);
}, "29-day M04 rejects 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months4, options).toPlainDateTime(),
  2019, 5, "M05", 30, 12, 34, 0, 0, 0, 0, "30-day M05 does not reject 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months5).toPlainDateTime(),
  2019, 6, "M06", 29, 12, 34, 0, 0, 0, 0, "29-day M06 constrains with addition");
assert.throws(RangeError, function () {
  date0130.add(months5, options);
}, "29-day M06 rejects 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months6).toPlainDateTime(),
  2019, 7, "M07", 29, 12, 34, 0, 0, 0, 0, "29-day M07 constrains with addition");
assert.throws(RangeError, function () {
  date0130.add(months6, options);
}, "29-day M07 rejects 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months7, options).toPlainDateTime(),
  2019, 8, "M08", 30, 12, 34, 0, 0, 0, 0, "30-day M08 does not reject 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months8).toPlainDateTime(),
  2019, 9, "M09", 29, 12, 34, 0, 0, 0, 0, "29-day M09 constrains with addition");
assert.throws(RangeError, function () {
  date0130.add(months8, options);
}, "29-day M09 rejects 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months9).toPlainDateTime(),
  2019, 10, "M10", 29, 12, 34, 0, 0, 0, 0, "29-day M10 constrains with addition");
assert.throws(RangeError, function () {
  date0130.add(months9, options);
}, "29-day M10 rejects 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months10, options).toPlainDateTime(),
  2019, 11, "M11", 30, 12, 34, 0, 0, 0, 0, "30-day M11 does not reject 30 with addition");

TemporalHelpers.assertPlainDateTime(
  date0130.add(months11, options).toPlainDateTime(),
  2019, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "30-day M12 does not reject 30 with addition");

// Months, backwards

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

TemporalHelpers.assertPlainDateTime(
  date1230.add(months1n).toPlainDateTime(),
  2022, 11, "M11", 29, 12, 34, 0, 0, 0, 0, "29-day M11 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.add(months1n, options);
}, "29-day M11 rejects 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months2n, options).toPlainDateTime(),
  2022, 10, "M10", 30, 12, 34, 0, 0, 0, 0, "30-day M10 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months3n).toPlainDateTime(),
  2022, 9, "M09", 29, 12, 34, 0, 0, 0, 0, "29-day M09 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.add(months3n, options);
}, "29-day M09 rejects 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months4n, options).toPlainDateTime(),
  2022, 8, "M08", 30, 12, 34, 0, 0, 0, 0, "30-day M08 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months5n).toPlainDateTime(),
  2022, 7, "M07", 29, 12, 34, 0, 0, 0, 0, "29-day M07 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.add(months5n, options);
}, "29-day M07 rejects 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months6n, options).toPlainDateTime(),
  2022, 6, "M06", 30, 12, 34, 0, 0, 0, 0, "30-day M06 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months7n, options).toPlainDateTime(),
  2022, 5, "M05", 30, 12, 34, 0, 0, 0, 0, "30-day M05 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months8n).toPlainDateTime(),
  2022, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "29-day M04 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.add(months8n, options);
}, "29-day M04 rejects 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months9n, options).toPlainDateTime(),
  2022, 3, "M03", 30, 12, 34, 0, 0, 0, 0, "30-day M03 does not reject 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months10n).toPlainDateTime(),
  2022, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "29-day M02 constrains with subtraction");
assert.throws(RangeError, function () {
  date1230.add(months10n, options);
}, "29-day M02 rejects 30 with subtraction");

TemporalHelpers.assertPlainDateTime(
  date1230.add(months11n, options).toPlainDateTime(),
  2022, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "30-day M01 does not reject 30 with subtraction");
