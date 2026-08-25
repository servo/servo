// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
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

const longLeapMonth = Temporal.PlainDate.from({ year: 2017, monthCode: "M06L", day: 30, calendar }, options);
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
  longLeapMonth.with({ year: 2025 }),
  2025, 7, "M06L", 29, "long M06L constrains to 29 when adjusting to year with short M06L");
assert.throws(RangeError, function () {
  longLeapMonth.with({ year: 2025 }, options);
}, "long M06L rejects when adjusting to year with short M06L");

// Constraining day 30 of each regular month to day 29

TemporalHelpers.assertPlainDate(
  date0130.with({ year: 2018 }),
  2018, 1, "M01", 29, "M01-30 constrains to 29");
assert.throws(RangeError, function () {
  date0130.with({ year: 2018 }, options);
}, "M01-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0230.with({ year: 2019 }),
  2019, 2, "M02", 29, "M02-30 constrains to 29");
assert.throws(RangeError, function () {
  date0230.with({ year: 2019 }, options);
}, "M02-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0330.with({ year: 2018 }),
  2018, 3, "M03", 29, "M03-30 constrains to 29");
assert.throws(RangeError, function () {
  date0330.with({ year: 2018 }, options);
}, "M03-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0430.with({ year: 2019 }),
  2019, 4, "M04", 29, "M04-30 constrains to 29");
assert.throws(RangeError, function () {
  date0430.with({ year: 2019 }, options);
}, "M04-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0530.with({ year: 2018 }),
  2018, 5, "M05", 29, "M05-30 constrains to 29");
assert.throws(RangeError, function () {
  date0530.with({ year: 2018 }, options);
}, "M05-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0630.with({ year: 2021 }),
  2021, 6, "M06", 29, "M06-30 constrains to 29");
assert.throws(RangeError, function () {
  date0630.with({ year: 2021 }, options);
}, "M06-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0730.with({ year: 2019 }),
  2019, 7, "M07", 29, "M07-30 constrains to 29");
assert.throws(RangeError, function () {
  date0730.with({ year: 2019 }, options);
}, "M07-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0830.with({ year: 2018 }),
  2018, 8, "M08", 29, "M08-30 constrains to 29");
assert.throws(RangeError, function () {
  date0830.with({ year: 2018 }, options);
}, "M08-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date0930.with({ year: 2022 }),
  2022, 9, "M09", 29, "M09-30 constrains to 29");
assert.throws(RangeError, function () {
  date0930.with({ year: 2022 }, options);
}, "M09-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date1030.with({ year: 2021 }),
  2021, 10, "M10", 29, "M10-30 constrains to 29");
assert.throws(RangeError, function () {
  date1030.with({ year: 2021 }, options);
}, "M10-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date1130.with({ year: 2022 }),
  2022, 11, "M11", 29, "M11-30 constrains to 29");
assert.throws(RangeError, function () {
  date1130.with({ year: 2022 }, options);
}, "M11-30 rejects in year with 29");

TemporalHelpers.assertPlainDate(
  date1230.with({ year: 2021 }),
  2021, 12, "M12", 29, "M12-30 constrains to 29");
assert.throws(RangeError, function () {
  date1230.with({ year: 2021 }, options);
}, "M12-30 rejects in year with 29");

// Months

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M02" }),
  2019, 2, "M02", 29, "29-day M02 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M02" }, options);
}, "29-day M02 rejects 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M03" }, options),
  2019, 3, "M03", 30, "30-day M03 does not reject 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M04" }),
  2019, 4, "M04", 29, "29-day M04 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M04" }, options);
}, "29-day M04 rejects 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M05" }, options),
  2019, 5, "M05", 30, "30-day M05 does not reject 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M06" }),
  2019, 6, "M06", 29, "29-day M06 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M06" }, options);
}, "29-day M06 rejects 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M07" }),
  2019, 7, "M07", 29, "29-day M07 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M07" }, options);
}, "29-day M07 rejects 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M08" }, options),
  2019, 8, "M08", 30, "30-day M08 does not reject 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M09" }),
  2019, 9, "M09", 29, "29-day M09 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M09" }, options);
}, "29-day M09 rejects 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M10" }),
  2019, 10, "M10", 29, "29-day M10 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M10" }, options);
}, "29-day M10 rejects 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M11" }, options),
  2019, 11, "M11", 30, "30-day M11 does not reject 30");

TemporalHelpers.assertPlainDate(
  date0130.with({ monthCode: "M12" }, options),
  2019, 12, "M12", 30, "30-day M12 does not reject 30");
