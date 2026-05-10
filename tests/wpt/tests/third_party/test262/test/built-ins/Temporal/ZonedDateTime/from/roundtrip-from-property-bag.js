// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Check that various dates created from a property bag have the expected
  properties
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };

const testData = [
  [2000, 1, "M01", 1],
  [1, 1, "M01", 1],
  [2021, 7, "M07", 15],
  [2021, 7, "M07", 3],
  [2021, 12, "M12", 31],
  [2021, 7, "M07", 15],
];

for (const [year, month, monthCode, day] of testData) {
  testRoundtrip(year, month, monthCode, day);
}

function testRoundtrip(year, month, monthCode, day) {
  const dateFromYearMonth = Temporal.ZonedDateTime.from({ year, month, day, hour: 12, minute: 34, second: 56, millisecond: 987, microsecond: 654, nanosecond: 321, timeZone: "UTC" }, options);
  TemporalHelpers.assertPlainDateTime(
    dateFromYearMonth.toPlainDateTime(),
    year, month, monthCode, day, 12, 34, 56, 987, 654, 321,
    `${dateFromYearMonth} - created from year and month`);

  const dateFromYearMonthCode = Temporal.ZonedDateTime.from({ year, monthCode, day, hour: 12, minute: 34, second: 56, millisecond: 987, microsecond: 654, nanosecond: 321, timeZone: "UTC" }, options);
  TemporalHelpers.assertPlainDateTime(
    dateFromYearMonthCode.toPlainDateTime(),
    year, month, monthCode, day, 12, 34, 56, 987, 654, 321,
    `${dateFromYearMonthCode} - created from year and month code`);
}
