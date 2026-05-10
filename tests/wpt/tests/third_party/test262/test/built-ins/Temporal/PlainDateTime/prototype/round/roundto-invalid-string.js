// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: RangeError thrown when smallestUnit option not one of the allowed string values
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 123, 987, 500);
const badValues = [
  "era",
  "eraYear",
  "year",
  "month",
  "week",
  "millisecond\0",
  "mill\u0131second",
  "SECOND",
  "eras",
  "eraYears",
  "years",
  "months",
  "weeks",
  "milliseconds\0",
  "mill\u0131seconds",
  "SECONDS",
  "other string",
];
for (const smallestUnit of badValues) {
  assert.throws(RangeError, () => datetime.round(smallestUnit),
    `"${smallestUnit}" is not a valid value for smallest unit`);
}
