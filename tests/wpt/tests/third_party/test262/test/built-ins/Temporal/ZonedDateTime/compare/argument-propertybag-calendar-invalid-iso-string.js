// Copyright (C) 2025 Brage Hogstad, University of Bergen. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Invalid ISO string as calendar should throw RangeError
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(0n, "UTC");

const invalidStrings = [
  ["", "empty string"],
  ["1997-12-04[u-ca=notacal]", "Unknown calendar"],
  ["notacal", "Unknown calendar"],
];

for (const [cal, description] of invalidStrings) {
  const arg = { year: 1976, monthCode: "M11", day: 18, calendar: cal, timeZone: "UTC" };
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare(arg, datetime),
    `${description} is not a valid calendar ID (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare(datetime, arg),
    `${description} is not a valid calendar ID (second argument)`
  );
}
