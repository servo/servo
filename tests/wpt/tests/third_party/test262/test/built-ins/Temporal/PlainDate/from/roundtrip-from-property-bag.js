// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: >
  Check that various dates created from a property bag have the expected
  properties
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const options = [
  { overflow: "constrain" },
  { overflow: "reject" }
]

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
  for (const opt of options) {
    const dateFromYearMonth = Temporal.PlainDate.from({ year, month, day }, opt);
    TemporalHelpers.assertPlainDate(
      dateFromYearMonth,
      year, month, monthCode, day,
      `${dateFromYearMonth} - created from year and month`);
  }

  for (const opt of options) {
    const dateFromYearMonthCode = Temporal.PlainDate.from({ year, monthCode, day }, opt);
    TemporalHelpers.assertPlainDate(
      dateFromYearMonthCode,
      year, month, monthCode, day,
      `${dateFromYearMonthCode} - created from year and month code`);
  }
}
