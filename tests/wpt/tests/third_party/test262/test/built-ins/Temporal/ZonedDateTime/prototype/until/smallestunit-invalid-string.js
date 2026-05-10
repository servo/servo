// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: RangeError thrown when smallestUnit option not one of the allowed string values
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");
const later = new Temporal.ZonedDateTime(1_000_090_061_987_654_321n, "UTC");
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
  "other string",
];
for (const smallestUnit of badValues) {
  assert.throws(RangeError, () => earlier.until(later, { smallestUnit }),
    `"${smallestUnit}" is not a valid value for smallest unit`);
}
