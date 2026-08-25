// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Constraining the day for 29/30-day months in islamic-umalqura calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-umalqura";
const options = { overflow: "reject" };

// Observational month lengths in AH year:
// Y \ M  1  2  3  4  5  6  7  8  9 10 11 12
// 1442: 29 30 29 30 29 30 29 30 30 29 30 29
// 1443: 30 29 30 29 30 29 30 29 30 29 30 30
// 1444: 29 30 29 30 30 29 29 30 29 30 29 30

// Years

const years1 = new Temporal.Duration(/* years = */ 1);
const years1n = new Temporal.Duration(-1);

for (const monthCode of ["M01", "M03", "M05", "M07", "M12"]) {
  const date1443 = Temporal.PlainDateTime.from({ year: 1443, monthCode, day: 30, hour: 12, minute: 34, calendar }, options);
  TemporalHelpers.assertPlainDateTime(
    date1443.add(years1n),
    1442, Number(monthCode.slice(1)), monthCode, 29, 12, 34, 0, 0, 0, 0, `Subtracting 1 year from ${monthCode}-30 into 29-day ${monthCode} constrains`,
    "ah", 1442
  );

  assert.throws(RangeError, function () {
    date1443.add(years1n, options);
  }, `Subtracting 1 year from ${monthCode}-30 into 29-day ${monthCode} rejects`);
}

for (const monthCode of ["M02", "M04", "M06", "M08"]) {
  const date1442 = Temporal.PlainDateTime.from({ year: 1442, monthCode, day: 30, hour: 12, minute: 34, calendar }, options);
  TemporalHelpers.assertPlainDateTime(
    date1442.add(years1),
    1443, Number(monthCode.slice(1)), monthCode, 29, 12, 34, 0, 0, 0, 0, `Adding 1 year to ${monthCode}-30 into 29-day ${monthCode} constrains`,
    "ah", 1443
  );

  assert.throws(RangeError, function () {
    date1442.add(years1, options);
  }, `Adding 1 year to ${monthCode}-30 into 29-day ${monthCode} rejects`);
}

for (const monthCode of ["M09", "M11"]) {
  const date1443 = Temporal.PlainDateTime.from({ year: 1443, monthCode, day: 30, hour: 12, minute: 34, calendar }, options);
  TemporalHelpers.assertPlainDateTime(
    date1443.add(years1),
    1444, Number(monthCode.slice(1)), monthCode, 29, 12, 34, 0, 0, 0, 0, `Adding 1 year to ${monthCode}-30 into 29-day ${monthCode} constrains`,
    "ah", 1444
  );

  assert.throws(RangeError, function () {
    date1443.add(years1, options);
  }, `Adding 1 year to ${monthCode}-30 into 29-day ${monthCode} rejects`);
}

const date1444 = Temporal.PlainDateTime.from({ year: 1444, monthCode: "M10", day: 30, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date1444.add(years1n),
  1443, 10, "M10", 29, 12, 34, 0, 0, 0, 0, `Subtracting 1 year from Shawwal 30 into 29-day Shawwal constrains`,
  "ah", 1443
);

assert.throws(RangeError, function () {
  date1444.add(years1n, options);
}, `Subtracting 1 year from Shawwal 30 into 29-day Shawwal rejects`);

// Months

const months1 = new Temporal.Duration(0, /* months = */ 1);
const months3 = new Temporal.Duration(0, 3);
const months5 = new Temporal.Duration(0, 5);
const months7 = new Temporal.Duration(0, 7);
const months8 = new Temporal.Duration(0, 8);
const months9 = new Temporal.Duration(0, 9);
const months10 = new Temporal.Duration(0, 10);
const months1n = new Temporal.Duration(0, -1);
const months3n = new Temporal.Duration(0, -3);
const months4n = new Temporal.Duration(0, -4);
const months6n = new Temporal.Duration(0, -6);
const months8n = new Temporal.Duration(0, -8);
const months10n = new Temporal.Duration(0, -10);

const date14420230 = Temporal.PlainDateTime.from({ year: 1442, monthCode: "M02", day: 30, hour: 12, minute: 34, calendar }, options);
const date14430130 = Temporal.PlainDateTime.from({ year: 1443, monthCode: "M01", day: 30, hour: 12, minute: 34, calendar }, options);
const date14440230 = Temporal.PlainDateTime.from({ year: 1444, monthCode: "M02", day: 30, hour: 12, minute: 34, calendar} , options);
const date14421130 = Temporal.PlainDateTime.from({ year: 1442, monthCode: "M11", day: 30, hour: 12, minute: 34, calendar }, options);
const date14431230 = Temporal.PlainDateTime.from({ year: 1443, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar }, options);
const date14441230 = Temporal.PlainDateTime.from({ year: 1444, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar} , options);

// Forwards

TemporalHelpers.assertPlainDateTime(
  date14431230.add(months1),
  1444, 1, "M01", 29, 12, 34, 0, 0, 0, 0, "29-day Muharram constrains",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14431230.add(months4n, options);
}, "29-day Muharram rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.add(months1),
  1443, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "29-day Safar constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14430130.add(months1, options);
}, "29-day Safar rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.add(months1),
  1442, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "29-day Rabi' al-Awwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.add(months1, options);
}, "29-day Rabi' al-Awwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.add(months3),
  1443, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "29-day Rabi' al-Thani constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14430130.add(months3, options);
}, "29-day Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.add(months3),
  1442, 5, "M05", 29, 12, 34, 0, 0, 0, 0, "29-day Jumada al-Awwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.add(months3, options);
}, "29-day Jumada al-Awwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.add(months5),
  1443, 6, "M06", 29, 12, 34, 0, 0, 0, 0, "29-day Jumada al-Thani constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14430130.add(months5, options);
}, "29-day Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.add(months5),
  1442, 7, "M07", 29, 12, 34, 0, 0, 0, 0, "29-day Rajab constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.add(months5, options);
}, "29-day Rajab rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.add(months7),
  1443, 8, "M08", 29, 12, 34, 0, 0, 0, 0, "29-day Sha'ban constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14430130.add(months7, options);
}, "29-day Sha'ban rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14440230.add(months7),
  1444, 9, "M09", 29, 12, 34, 0, 0, 0, 0, "29-day Ramadan constrains",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14440230.add(months7, options);
}, "29-day Ramadan rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.add(months8),
  1442, 10, "M10", 29, 12, 34, 0, 0, 0, 0, "29-day Shawwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.add(months8, options);
}, "29-day Shawwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14440230.add(months9),
  1444, 11, "M11", 29, 12, 34, 0, 0, 0, 0, "29-day Dhu al-Qadah constrains",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14440230.add(months9, options);
}, "29-day Dhu al-Qadah rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.add(months10),
  1442, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "29-day Dhu al-Hijjah constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.add(months10, options);
}, "29-day Dhu al-Hijjah rejects with 30");

// Backwards

TemporalHelpers.assertPlainDateTime(
  date14421130.add(months10n),
  1442, 1, "M01", 29, 12, 34, 0, 0, 0, 0, "29-day Muharram constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14421130.add(months10n, options);
}, "29-day Muharram rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14431230.add(months10n),
  1443, 2, "M02", 29, 12, 34, 0, 0, 0, 0, "29-day Safar constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14431230.add(months10n, options);
}, "29-day Safar rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14421130.add(months8n),
  1442, 3, "M03", 29, 12, 34, 0, 0, 0, 0, "29-day Rabi' al-Awwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14421130.add(months8n, options);
}, "29-day Rabi' al-Awwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14431230.add(months8n),
  1443, 4, "M04", 29, 12, 34, 0, 0, 0, 0, "29-day Rabi' al-Thani constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14431230.add(months8n, options);
}, "29-day Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14421130.add(months6n),
  1442, 5, "M05", 29, 12, 34, 0, 0, 0, 0, "29-day Jumada al-Awwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14421130.add(months6n, options);
}, "29-day Jumada al-Awwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14431230.add(months6n),
  1443, 6, "M06", 29, 12, 34, 0, 0, 0, 0, "29-day Jumada al-Thani constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14431230.add(months6n, options);
}, "29-day Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14421130.add(months4n),
  1442, 7, "M07", 29, 12, 34, 0, 0, 0, 0, "29-day Rajab constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14421130.add(months4n, options);
}, "29-day Rajab rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14431230.add(months4n),
  1443, 8, "M08", 29, 12, 34, 0, 0, 0, 0, "29-day Sha'ban constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14431230.add(months4n, options);
}, "29-day Sha'ban rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14441230.add(months3n),
  1444, 9, "M09", 29, 12, 34, 0, 0, 0, 0, "29-day Ramadan constrains",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14441230.add(months3n, options);
}, "29-day Ramadan rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14421130.add(months1n),
  1442, 10, "M10", 29, 12, 34, 0, 0, 0, 0, "29-day Shawwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14421130.add(months1n, options);
}, "29-day Shawwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14441230.add(months1n),
  1444, 11, "M11", 29, 12, 34, 0, 0, 0, 0, "29-day Dhu al-Qadah constrains",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14441230.add(months1n, options);
}, "29-day Dhu al-Qadah rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.add(months1n),
  1442, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "29-day Dhu al-Hijjah constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14430130.add(months1n, options);
}, "29-day Dhu al-Hijjah rejects with 30");
