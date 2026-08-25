// Copyright (C) 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: >
  RangeError thrown if an invalid ISO string (or syntactically valid ISO string
  that is not supported) is used as an Instant
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  // invalid ISO strings:
  "",
  "invalid iso8601",
  "2020-01-00T00:00Z",
  "2020-01-32T00:00Z",
  "2020-02-30T00:00Z",
  "2021-02-29T00:00Z",
  "2020-00-01T00:00Z",
  "2020-13-01T00:00Z",
  "2020-01-01TZ",
  "2020-01-01T25:00:00Z",
  "2020-01-01T01:60:00Z",
  "2020-01-01T01:60:61Z",
  "2020-01-01T00:00Zjunk",
  "2020-01-01T00:00:00Zjunk",
  "2020-01-01T00:00:00.000000000Zjunk",
  "2020-01-01T00:00:00+00:00junk",
  "2020-01-01T00:00:00+00:00[UTC]junk",
  "2020-01-01T00:00:00+00:00[UTC][u-ca=iso8601]junk",
  "02020-01-01T00:00Z",
  "2020-001-01T00:00Z",
  "2020-01-001T00:00Z",
  "2020-01-01T001Z",
  "2020-01-01T01:001Z",
  "2020-01-01T01:01:001Z",
  "2020-01-01T00:00-24:00",
  "2020-01-01T00:00+24:00",
  // valid, but forms not supported in Temporal:
  "2020-W01-1T00:00Z",
  "2020-001T00:00Z",
  "+0002020-01-01T00:00Z",
  // may be valid in other contexts, but insufficient information for Instant:
  "2020-01",
  "+002020-01",
  "01-01",
  "2020-W01",
  "P1Y",
  "-P12Y",
  "2020-01-01",
  "2020-01-01T00",
  "2020-01-01T00:00",
  "2020-01-01T00:00:00",
  "2020-01-01T00:00:00.000000000",
  // valid, but outside the supported range:
  "-999999-01-01T00:00Z",
  "+999999-01-01T00:00Z",
];

const instance = new Temporal.Instant(0n);
for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => instance.since(arg),
    `"${arg}" should not be a valid ISO string for an Instant`
  );
}
