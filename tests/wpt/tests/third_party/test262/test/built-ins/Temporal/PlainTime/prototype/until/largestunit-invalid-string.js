// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: RangeError thrown when largestUnit option not one of the allowed string values
features: [Temporal]
---*/

const earlier = new Temporal.PlainTime(12, 34, 56, 0, 0, 0);
const later = new Temporal.PlainTime(13, 35, 57, 987, 654, 321);
const badValues = [
  "era",
  "eraYear",
  "year",
  "month",
  "week",
  "day",
  "millisecond\0",
  "mill\u0131second",
  "SECOND",
  "eras",
  "eraYears",
  "years",
  "months",
  "weeks",
  "days",
  "milliseconds\0",
  "mill\u0131seconds",
  "SECONDS",
  "other string"
];
for (const largestUnit of badValues) {
  assert.throws(RangeError, () => earlier.until(later, { largestUnit }),
    `"${largestUnit}" is not a valid value for largestUnit`);
}
