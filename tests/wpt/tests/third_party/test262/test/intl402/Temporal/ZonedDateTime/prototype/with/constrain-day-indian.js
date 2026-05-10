// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
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

// Common year

[
  [3, "M03"],
  [4, "M04"],
  [5, "M05"],
  [6, "M06"],
].forEach(function ([month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    common0231.with({ monthCode }, options).toPlainDateTime(),
    1944, month, monthCode, 31,  12, 34, 0, 0, 0, 0,`common-year ${monthCode} does not reject 31 when adding`,
    "shaka", 1944);
});

[
  [1, "M01"],
  [7, "M07"],
  [8, "M08"],
  [9, "M09"],
  [10, "M10"],
  [11, "M11"],
  [12, "M12"],
].forEach(function ([month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    common0231.with({ monthCode }).toPlainDateTime(),
    1944, month, monthCode, 30,  12, 34, 0, 0, 0, 0,`common-year ${monthCode} constrains to 30 when adding`,
    "shaka", 1944);
  assert.throws(RangeError, function () {
    common0231.with({ monthCode }, options);
  }, `common-year ${monthCode} rejects 31 when adding`);
});

// Leap year

[
  [2, "M02"],
  [3, "M03"],
  [4, "M04"],
  [5, "M05"],
  [6, "M06"],
].forEach(function ([month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    leap0131.with({ monthCode }, options).toPlainDateTime(),
    1946, month, monthCode, 31,  12, 34, 0, 0, 0, 0,`leap-year ${monthCode} does not reject 31 when adding`,
    "shaka", 1946);
});

[
  [7, "M07"],
  [8, "M08"],
  [9, "M09"],
  [10, "M10"],
  [11, "M11"],
  [12, "M12"],
].forEach(function ([month, monthCode]) {
  TemporalHelpers.assertPlainDateTime(
    leap0131.with({ monthCode }).toPlainDateTime(),
    1946, month, monthCode, 30,  12, 34, 0, 0, 0, 0,`leap-year ${monthCode} constrains to 30 when adding`,
    "shaka", 1946);
  assert.throws(RangeError, function () {
    leap0131.with({ monthCode }, options);
  }, `leap-year ${monthCode} rejects 31 when adding`);
});
