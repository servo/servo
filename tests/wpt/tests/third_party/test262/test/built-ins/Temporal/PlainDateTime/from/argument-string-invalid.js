// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Reject string argument if it cannot be parsed
features: [Temporal]
---*/

const invalidStrings = [
  // invalid ISO strings:
  "",
  "obviously invalid",
  "2020-01-00",
  "2020-01-32",
  "2020-02-30",
  "2021-02-29",
  "2020-00-01",
  "2020-13-01",
  "2020-01-01T",
  "2020-01-01T25:00:00",
  "2020-01-01T01:60:00",
  "2020-01-01T01:60:61",
  "2020-01-01junk",
  "2020-01-01T00:00:00junk",
  "2020-01-01T00:00:00+00:00junk",
  "2020-01-01T00:00:00+00:00[UTC]junk",
  "2020-01-01T00:00:00+00:00[UTC][u-ca=iso8601]junk",
  "02020-01-01",
  "2020-001-01",
  "2020-01-001",
  "2020-01-01T001",
  "2020-01-01T01:001",
  "2020-01-01T01:01:001",
  // valid, but forms not supported in Temporal:
  "2020-W01-1",
  "2020-001",
  "+0002020-01-01",
  // valid, but this calendar must not exist:
  "2020-01-01[u-ca=notexist]",
  // may be valid in other contexts, but insufficient information for PlainDateTime:
  "2020-01",
  "+002020-01",
  "01-01",
  "2020-W01",
  "P1Y",
  "-P12Y",
  // valid, but outside the supported range:
  "-999999-01-01",
  "+999999-01-01",
  // "00:0000" is invalid (the hour/minute and minute/second separator
  // or lack thereof needs to match).
  "2025-01-01T00:00:00+00:0000",
  "2025-01-01T00:00:00+0000:00",
  "202501-01T00:00:00",
  "2025-0101T00:00:00",
  "2025-01-01T00:0000",
  "2025-01-01T0000:00",
];

invalidStrings.forEach((s) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainDateTime.from(s),
    `invalid date-time string (${s})`
  );
});
