// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: RangeError thrown when smallestUnit option not one of the allowed string values
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_123_987_500n);
const badValues = [
  "era",
  "eraYear",
  "year",
  "month",
  "week",
  "day",
  "hour",
  "millisecond\0",
  "mill\u0131second",
  "SECOND",
  "eras",
  "eraYears",
  "years",
  "months",
  "weeks",
  "days",
  "hours",
  "milliseconds\0",
  "mill\u0131seconds",
  "SECONDS",
  "other string",
];
for (const smallestUnit of badValues) {
  assert.throws(RangeError, () => instant.toString({ smallestUnit }),
    `"${smallestUnit}" is not a valid value for smallest unit`);
}
