// Copyright (C) 2025 Brage Hogstad, University of Bergen. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.constructor
description: Invalid ISO string as calendar should throw RangeError
features: [Temporal]
---*/

const invalidStrings = [
  ["", "empty string"],
  ["1997-12-04[u-ca=iso8601]", "ISO string with calendar annotation"],
  ["notacal", "Unknown calendar"],
  ["11111111", "compact ISO date used as calendar name"],
  ["1111-11-11", "extended ISO date used as calendar name"],
];

for (const [arg, description] of invalidStrings) {
  assert.throws(
    RangeError,
    () => new Temporal.PlainYearMonth(2000, 5, arg, 1),
    `${description} is not a valid calendar ID`
  );
}
