// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: RangeError thrown when smallestUnit option not one of the allowed string values
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 123, 987, 500);
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
  "other string",
];
for (const smallestUnit of badValues) {
  assert.throws(RangeError, () => time.round({ smallestUnit }),
    `"${smallestUnit}" is not a valid value for smallest unit`);
}
