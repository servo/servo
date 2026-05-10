// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
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

for (const monthCode of ["M01", "M03", "M05", "M07", "M12"]) {
  const date1443 = Temporal.PlainDateTime.from({ year: 1443, monthCode, day: 30, hour: 12, minute: 34, calendar }, options);
  TemporalHelpers.assertPlainDateTime(
    date1443.with({ year: 1442 }),
    1442, Number(monthCode.slice(1)), monthCode, 29,  12, 34, 0, 0, 0, 0,`Changing the year from ${monthCode}-30 into 29-day ${monthCode} constrains`,
    "ah", 1442
  );

  assert.throws(RangeError, function () {
    date1443.with({ year: 1442 }, options);
  }, `Changing the year from ${monthCode}-30 into 29-day ${monthCode} rejects`);
}

for (const monthCode of ["M02", "M04", "M06", "M08"]) {
  const date1442 = Temporal.PlainDateTime.from({ year: 1442, monthCode, day: 30, hour: 12, minute: 34, calendar }, options);
  TemporalHelpers.assertPlainDateTime(
    date1442.with({ year: 1443 }),
    1443, Number(monthCode.slice(1)), monthCode, 29,  12, 34, 0, 0, 0, 0,`Changing the year from ${monthCode}-30 into 29-day ${monthCode} constrains`,
    "ah", 1443
  );

  assert.throws(RangeError, function () {
    date1442.with({ year: 1443 }, options);
  }, `Changing the year from ${monthCode}-30 into 29-day ${monthCode} rejects`);
}

for (const monthCode of ["M09", "M11"]) {
  const date1443 = Temporal.PlainDateTime.from({ year: 1443, monthCode, day: 30, hour: 12, minute: 34, calendar }, options);
  TemporalHelpers.assertPlainDateTime(
    date1443.with({ year: 1444 }),
    1444, Number(monthCode.slice(1)), monthCode, 29,  12, 34, 0, 0, 0, 0,`Changing the year from ${monthCode}-30 into 29-day ${monthCode} constrains`,
    "ah", 1444
  );

  assert.throws(RangeError, function () {
    date1443.with({ year: 1444 }, options);
  }, `Changing the year from ${monthCode}-30 into 29-day ${monthCode} rejects`);
}

const date1444 = Temporal.PlainDateTime.from({ year: 1444, monthCode: "M10", day: 30, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date1444.with({ year: 1443 }),
  1443, 10, "M10", 29,  12, 34, 0, 0, 0, 0,`Changing the year from Shawwal 30 into 29-day Shawwal constrains`,
  "ah", 1443
);

assert.throws(RangeError, function () {
  date1444.with({ year: 1443 }, options);
}, `Changing the year from Shawwal 30 into 29-day Shawwal rejects`);

// Months

const date14420230 = Temporal.PlainDateTime.from({ year: 1442, monthCode: "M02", day: 30, hour: 12, minute: 34, calendar }, options);
const date14430130 = Temporal.PlainDateTime.from({ year: 1443, monthCode: "M01", day: 30, hour: 12, minute: 34, calendar }, options);
const date14441230 = Temporal.PlainDateTime.from({ year: 1444, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar} , options);

// Forwards

TemporalHelpers.assertPlainDateTime(
  date14441230.with({ monthCode: "M01" }),
  1444, 1, "M01", 29,  12, 34, 0, 0, 0, 0,"29-day Muharram constrains",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14441230.with({ monthCode: "M01" }, options);
}, "29-day Muharram rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.with({ monthCode: "M02" }),
  1443, 2, "M02", 29,  12, 34, 0, 0, 0, 0,"29-day Safar constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14430130.with({ monthCode: "M02" }, options);
}, "29-day Safar rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.with({ monthCode: "M03" }),
  1442, 3, "M03", 29,  12, 34, 0, 0, 0, 0,"29-day Rabi' al-Awwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.with({ monthCode: "M03" }, options);
}, "29-day Rabi' al-Awwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.with({ monthCode: "M04" }),
  1443, 4, "M04", 29,  12, 34, 0, 0, 0, 0,"29-day Rabi' al-Thani constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14430130.with({ monthCode: "M04" }, options);
}, "29-day Rabi' al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.with({ monthCode: "M05" }),
  1442, 5, "M05", 29,  12, 34, 0, 0, 0, 0,"29-day Jumada al-Awwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.with({ monthCode: "M05" }, options);
}, "29-day Jumada al-Awwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.with({ monthCode: "M06" }),
  1443, 6, "M06", 29,  12, 34, 0, 0, 0, 0,"29-day Jumada al-Thani constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14430130.with({ monthCode: "M06" }, options);
}, "29-day Jumada al-Thani rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.with({ monthCode: "M07" }),
  1442, 7, "M07", 29,  12, 34, 0, 0, 0, 0,"29-day Rajab constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.with({ monthCode: "M07" }, options);
}, "29-day Rajab rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14430130.with({ monthCode: "M08" }),
  1443, 8, "M08", 29,  12, 34, 0, 0, 0, 0,"29-day Sha'ban constrains",
  "ah", 1443);
assert.throws(RangeError, function () {
  date14430130.with({ monthCode: "M08" }, options);
}, "29-day Sha'ban rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14441230.with({ monthCode: "M09" }),
  1444, 9, "M09", 29,  12, 34, 0, 0, 0, 0,"29-day Ramadan constrains",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14441230.with({ monthCode: "M09" }, options);
}, "29-day Ramadan rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.with({ monthCode: "M10" }),
  1442, 10, "M10", 29,  12, 34, 0, 0, 0, 0,"29-day Shawwal constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.with({ monthCode: "M10" }, options);
}, "29-day Shawwal rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14441230.with({ monthCode: "M11" }),
  1444, 11, "M11", 29,  12, 34, 0, 0, 0, 0,"29-day Dhu al-Qadah constrains",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14441230.with({ monthCode: "M11" }, options);
}, "29-day Dhu al-Qadah rejects with 30");

TemporalHelpers.assertPlainDateTime(
  date14420230.with({ monthCode: "M12" }),
  1442, 12, "M12", 29,  12, 34, 0, 0, 0, 0,"29-day Dhu al-Hijjah constrains",
  "ah", 1442);
assert.throws(RangeError, function () {
  date14420230.with({ monthCode: "M12" }, options);
}, "29-day Dhu al-Hijjah rejects with 30");
