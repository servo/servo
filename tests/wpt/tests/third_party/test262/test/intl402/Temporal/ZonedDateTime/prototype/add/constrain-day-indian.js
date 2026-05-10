// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Constraining the day at end of month (indian calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "indian";
const options = { overflow: "reject" };

// 31-day months: 02-06
// 30-day months: 07-12
// Chaitra (01) has 30 days in common years and 31 in leap years
// See leap-year-indian.js for tests adding years

const common0231 = Temporal.ZonedDateTime.from({ year: 1944, monthCode: "M02", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const leap0131 = Temporal.ZonedDateTime.from({ year: 1946, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const common0231After = Temporal.ZonedDateTime.from({ year: 1947, monthCode: "M02", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

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
const months12n = new Temporal.Duration(0, -12);
const months13n = new Temporal.Duration(0, -13);

// Common year, forwards

[
  [months1, 3, "M03"],
  [months2, 4, "M04"],
  [months3, 5, "M05"],
  [months4, 6, "M06"],
].forEach(function ([months, month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    common0231.add(months, options).toPlainDateTime(),
    1944, month, monthCode, 31, 12, 34, 0, 0, 0, 0, `common-year ${monthCode} does not reject 31 when adding`,
    "shaka", 1944);
});

[
  [months5, 7, "M07"],
  [months6, 8, "M08"],
  [months7, 9, "M09"],
  [months8, 10, "M10"],
  [months9, 11, "M11"],
  [months10, 12, "M12"],
].forEach(function ([months, month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    common0231.add(months).toPlainDateTime(),
    1944, month, monthCode, 30, 12, 34, 0, 0, 0, 0, `common-year ${monthCode} constrains to 30 when adding`,
    "shaka", 1944);
  assert.throws(RangeError, function () {
    common0231.add(months, options);
  }, `common-year ${monthCode} rejects 31 when adding`);
});

TemporalHelpers.assertPlainDateTime(
  common0231.add(months11).toPlainDateTime(),
  1945, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "common-year Chaitra constrains to 30 when adding",
  "shaka", 1945);
assert.throws(RangeError, function () {
  common0231.add(months11, options);
}, "common-year Chaitra rejects 31 when adding");

// Leap year, forwards

[
  [months1, 2, "M02"],
  [months2, 3, "M03"],
  [months3, 4, "M04"],
  [months4, 5, "M05"],
  [months5, 6, "M06"],
].forEach(function ([months, month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    leap0131.add(months, options).toPlainDateTime(),
    1946, month, monthCode, 31, 12, 34, 0, 0, 0, 0, `leap-year ${monthCode} does not reject 31 when adding`,
    "shaka", 1946);
});

[
  [months6, 7, "M07"],
  [months7, 8, "M08"],
  [months8, 9, "M09"],
  [months9, 10, "M10"],
  [months10, 11, "M11"],
  [months11, 12, "M12"],
].forEach(function ([months, month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    leap0131.add(months).toPlainDateTime(),
    1946, month, monthCode, 30, 12, 34, 0, 0, 0, 0, `leap-year ${monthCode} constrains to 30 when adding`,
    "shaka", 1946);
  assert.throws(RangeError, function () {
    leap0131.add(months, options);
  }, `leap-year ${monthCode} rejects 31 when adding`);
});

// Common year, backwards

TemporalHelpers.assertPlainDateTime(
  common0231.add(months1n).toPlainDateTime(),
  1944, 1, "M01", 30, 12, 34, 0, 0, 0, 0, `common-year Chaitra constrains to 30 when subtracting`,
  "shaka", 1944);
assert.throws(RangeError, function () {
  common0231.add(months1n, options);
}, "common-year Chaitra rejects 31 when subtracting");

[
  [months12n, 2, "M02"],
  [months11n, 3, "M03"],
  [months10n, 4, "M04"],
  [months9n, 5, "M05"],
  [months8n, 6, "M06"]
].forEach(function ([months, month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    common0231.add(months, options).toPlainDateTime(),
    1943, month, monthCode, 31, 12, 34, 0, 0, 0, 0, `common-year ${monthCode} does not reject 31 when subtracting`,
    "shaka", 1943);
});

[
  [months7n, 7, "M07"],
  [months6n, 8, "M08"],
  [months5n, 9, "M09"],
  [months4n, 10, "M10"],
  [months3n, 11, "M11"],
  [months2n, 12, "M12"],
].forEach(function ([months, month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    common0231.add(months).toPlainDateTime(),
    1943, month, monthCode, 30, 12, 34, 0, 0, 0, 0, `common-year ${monthCode} constrains to 30 when subtracting`,
    "shaka", 1943);
  assert.throws(RangeError, function () {
    common0231.add(months, options);
  }, `common-year ${monthCode} rejects 31 when adding`);
});

// Leap year, backwards

[
  [months13n, 1, "M01"],
  [months12n, 2, "M02"],
  [months11n, 3, "M03"],
  [months10n, 4, "M04"],
  [months9n, 5, "M05"],
  [months8n, 6, "M06"],
].forEach(function ([months, month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    common0231After.add(months, options).toPlainDateTime(),
    1946, month, monthCode, 31, 12, 34, 0, 0, 0, 0, `leap-year ${monthCode} does not reject 31 when subtracting`,
    "shaka", 1946);
});

[
  [months7n, 7, "M07"],
  [months6n, 8, "M08"],
  [months5n, 9, "M09"],
  [months4n, 10, "M10"],
  [months3n, 11, "M11"],
  [months2n, 12, "M12"],
].forEach(function ([months, month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    common0231After.add(months).toPlainDateTime(),
    1946, month, monthCode, 30, 12, 34, 0, 0, 0, 0, `leap-year ${monthCode} constrains to 30 when subtracting`,
    "shaka", 1946);
  assert.throws(RangeError, function () {
    common0231After.add(months, options);
  }, `leap-year ${monthCode} rejects 31 when adding`);
});
