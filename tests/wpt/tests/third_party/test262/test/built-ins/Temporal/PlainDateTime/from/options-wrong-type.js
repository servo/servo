// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
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
  Infinity,
  NaN,
  null,
];

for (const value of badOptions) {
  assert.throws(TypeError, () => Temporal.PlainDateTime.from({ year: 1976, month: 11, day: 18 }, value),
    `TypeError on wrong options type ${typeof value}`);
  assert.throws(TypeError, () => Temporal.PlainDateTime.from(new Temporal.PlainDateTime(1976, 11, 18), value),
      "TypeError thrown before cloning PlainDateTime instance");
  assert.throws(TypeError, () => Temporal.PlainDateTime.from(new Temporal.ZonedDateTime(0n, "UTC"), value),
    "TypeError thrown before converting ZonedDateTime instance");
  assert.throws(TypeError, () => Temporal.PlainDateTime.from(new Temporal.PlainDate(1976, 11, 18), value),
    "TypeError thrown before converting PlainDate instance");
  assert.throws(RangeError, () => Temporal.PlainDateTime.from("1976-11-18T12:34Z", value),
    "Invalid string processed before throwing TypeError");
};
