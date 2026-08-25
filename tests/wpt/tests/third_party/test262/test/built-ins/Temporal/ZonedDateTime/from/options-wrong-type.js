// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: TypeError thrown when options argument is a primitive
features: [BigInt, Symbol, Temporal]
---*/

const badOptions = [
  null,
  true,
  "some string",
  Symbol(),
  1,
  2n,
];

for (const value of badOptions) {
  assert.throws(TypeError, () => Temporal.ZonedDateTime.from({ year: 1976, month: 11, day: 18, timeZone: "UTC" }, value),
    `TypeError on wrong options type ${typeof value}`);
  assert.throws(TypeError, () => Temporal.ZonedDateTime.from(new Temporal.ZonedDateTime(0n, "UTC"), value),
    "TypeError thrown before cloning ZonedDateTime instance");
  assert.throws(RangeError, () => Temporal.ZonedDateTime.from("1976-11-18Z", value),
    "Invalid string string processed before throwing TypeError");
};
