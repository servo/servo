// Copyright (C) 2025 Brage Hogstad, University of Bergen. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Invalid ISO string as calendar should throw RangeError
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

const invalidStrings = [
  ["", "empty string"],
  ["1997-12-04[u-ca=notacal]", "Unknown calendar"],
  ["notacal", "Unknown calendar"],
];

for (const [cal, description] of invalidStrings) {
  const arg = { year: 1970, monthCode: "M11", day: 18, calendar: cal };
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `${description} is not a valid calendar ID`
  );
}
