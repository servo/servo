// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
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

const longLeapMonth = Temporal.ZonedDateTime.from({ year: 2012, monthCode: "M03L", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
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
  longLeapMonth.with({ year: 1993 }).toPlainDateTime(),
  1993, 4, "M03L", 29,  12, 34, 0, 0, 0, 0,"long M03L constrains to 29 when adjusting to year with short M03L");
assert.throws(RangeError, function () {
  longLeapMonth.with({ year: 1993 }, options);
}, "long M06L rejects when adjusting to year with short M06L");

// Constraining day 30 of each regular month to day 29

TemporalHelpers.assertPlainDateTime(
  date0130.with({ year: 2018 }).toPlainDateTime(),
  2018, 1, "M01", 29,  12, 34, 0, 0, 0, 0,"M01-30 constrains to 29");
assert.throws(RangeError, function () {
  date0130.with({ year: 2018 }, options);
}, "M01-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0230.with({ year: 2019 }).toPlainDateTime(),
  2019, 2, "M02", 29,  12, 34, 0, 0, 0, 0,"M02-30 constrains to 29");
assert.throws(RangeError, function () {
  date0230.with({ year: 2019 }, options);
}, "M02-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0330.with({ year: 2018 }).toPlainDateTime(),
  2018, 3, "M03", 29,  12, 34, 0, 0, 0, 0,"M03-30 constrains to 29");
assert.throws(RangeError, function () {
  date0330.with({ year: 2018 }, options);
}, "M03-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0430.with({ year: 2019 }).toPlainDateTime(),
  2019, 4, "M04", 29,  12, 34, 0, 0, 0, 0,"M04-30 constrains to 29");
assert.throws(RangeError, function () {
  date0430.with({ year: 2019 }, options);
}, "M04-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0530.with({ year: 2018 }).toPlainDateTime(),
  2018, 5, "M05", 29,  12, 34, 0, 0, 0, 0,"M05-30 constrains to 29");
assert.throws(RangeError, function () {
  date0530.with({ year: 2018 }, options);
}, "M05-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0630.with({ year: 2021 }).toPlainDateTime(),
  2021, 6, "M06", 29,  12, 34, 0, 0, 0, 0,"M06-30 constrains to 29");
assert.throws(RangeError, function () {
  date0630.with({ year: 2021 }, options);
}, "M06-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0730.with({ year: 2019 }).toPlainDateTime(),
  2019, 7, "M07", 29,  12, 34, 0, 0, 0, 0,"M07-30 constrains to 29");
assert.throws(RangeError, function () {
  date0730.with({ year: 2019 }, options);
}, "M07-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0830.with({ year: 2018 }).toPlainDateTime(),
  2018, 8, "M08", 29,  12, 34, 0, 0, 0, 0,"M08-30 constrains to 29");
assert.throws(RangeError, function () {
  date0830.with({ year: 2018 }, options);
}, "M08-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date0930.with({ year: 2022 }).toPlainDateTime(),
  2022, 9, "M09", 29,  12, 34, 0, 0, 0, 0,"M09-30 constrains to 29");
assert.throws(RangeError, function () {
  date0930.with({ year: 2022 }, options);
}, "M09-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date1030.with({ year: 2021 }).toPlainDateTime(),
  2021, 10, "M10", 29,  12, 34, 0, 0, 0, 0,"M10-30 constrains to 29");
assert.throws(RangeError, function () {
  date1030.with({ year: 2021 }, options);
}, "M10-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date1130.with({ year: 2022 }).toPlainDateTime(),
  2022, 11, "M11", 29,  12, 34, 0, 0, 0, 0,"M11-30 constrains to 29");
assert.throws(RangeError, function () {
  date1130.with({ year: 2022 }, options);
}, "M11-30 rejects in year with 29");

TemporalHelpers.assertPlainDateTime(
  date1230.with({ year: 2021 }).toPlainDateTime(),
  2021, 12, "M12", 29,  12, 34, 0, 0, 0, 0,"M12-30 constrains to 29");
assert.throws(RangeError, function () {
  date1230.with({ year: 2021 }, options);
}, "M12-30 rejects in year with 29");

// Months

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M02" }).toPlainDateTime(),
  2019, 2, "M02", 29,  12, 34, 0, 0, 0, 0,"29-day M02 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M02" }, options);
}, "29-day M02 rejects 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M03" }, options).toPlainDateTime(),
  2019, 3, "M03", 30,  12, 34, 0, 0, 0, 0,"30-day M03 does not reject 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M04" }).toPlainDateTime(),
  2019, 4, "M04", 29,  12, 34, 0, 0, 0, 0,"29-day M04 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M04" }, options);
}, "29-day M04 rejects 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M05" }, options).toPlainDateTime(),
  2019, 5, "M05", 30,  12, 34, 0, 0, 0, 0,"30-day M05 does not reject 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M06" }).toPlainDateTime(),
  2019, 6, "M06", 29,  12, 34, 0, 0, 0, 0,"29-day M06 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M06" }, options);
}, "29-day M06 rejects 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M07" }).toPlainDateTime(),
  2019, 7, "M07", 29,  12, 34, 0, 0, 0, 0,"29-day M07 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M07" }, options);
}, "29-day M07 rejects 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M08" }, options).toPlainDateTime(),
  2019, 8, "M08", 30,  12, 34, 0, 0, 0, 0,"30-day M08 does not reject 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M09" }).toPlainDateTime(),
  2019, 9, "M09", 29,  12, 34, 0, 0, 0, 0,"29-day M09 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M09" }, options);
}, "29-day M09 rejects 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M10" }, options).toPlainDateTime(),
  2019, 10, "M10", 30,  12, 34, 0, 0, 0, 0,"30-day M10 does not reject 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M11" }).toPlainDateTime(),
  2019, 11, "M11", 29,  12, 34, 0, 0, 0, 0,"29-day M11 constrains");
assert.throws(RangeError, function () {
  date0130.with({ monthCode: "M11" }, options);
}, "29-day M11 rejects 30");

TemporalHelpers.assertPlainDateTime(
  date0130.with({ monthCode: "M12" }, options).toPlainDateTime(),
  2019, 12, "M12", 30,  12, 34, 0, 0, 0, 0,"30-day M12 does not reject 30");
