// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: RangeError thrown when largestUnit option not one of the allowed string values
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 0, 0, 0);
const later = new Temporal.PlainDateTime(2001, 6, 3, 13, 35, 57, 987, 654, 321);
const badValues = [
  "era",
  "eraYear",
  "millisecond\0",
  "mill\u0131second",
  "SECOND",
  "eras",
  "eraYears",
  "milliseconds\0",
  "mill\u0131seconds",
  "SECONDS",
  "other string"
];
for (const largestUnit of badValues) {
  assert.throws(RangeError, () => earlier.until(later, { largestUnit }),
    `"${largestUnit}" is not a valid value for largestUnit`);
}
