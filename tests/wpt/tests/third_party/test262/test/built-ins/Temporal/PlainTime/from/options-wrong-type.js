// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
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
  assert.throws(TypeError, () => Temporal.PlainTime.from({ hour: 12, minute: 34 }, value),
    `TypeError on wrong options type ${typeof value}`);
  assert.throws(TypeError, () => Temporal.PlainTime.from(new Temporal.PlainTime(12, 34), value),
    "TypeError thrown before cloning PlainTime instance");
  assert.throws(TypeError, () => Temporal.PlainTime.from(new Temporal.ZonedDateTime(0n, "UTC"), value),
    "TypeError thrown before converting ZonedDateTime instance");
  assert.throws(TypeError, () => Temporal.PlainTime.from(new Temporal.PlainDateTime(1976, 11, 18), value),
    "TypeError thrown before converting PlainDateTime instance");
  assert.throws(RangeError, () => Temporal.PlainTime.from("T99:99", value),
    "Invalid string processed before throwing TypeError");
};
