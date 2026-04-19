// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: RangeError thrown when smallestUnit option not one of the allowed string values
features: [Temporal]
---*/

const earlier = new Temporal.PlainDate(2000, 5, 2);
const later = new Temporal.PlainDate(2001, 6, 3);
const badValues = [
  "era",
  "eraYear",
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond",
  "nanosecond",
  "month\0",
  "YEAR",
  "eras",
  "eraYears",
  "hours",
  "minutes",
  "seconds",
  "milliseconds",
  "microseconds",
  "nanoseconds",
  "months\0",
  "YEARS",
  "other string",
];
for (const smallestUnit of badValues) {
  assert.throws(RangeError, () => later.since(earlier, { smallestUnit }),
    `"${smallestUnit}" is not a valid value for smallest unit`);
}
