// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
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
  assert.throws(TypeError, () => Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M01" }, value),
    `TypeError on wrong options type ${typeof value}`);
  assert.throws(TypeError, () => Temporal.PlainYearMonth.from(new Temporal.PlainYearMonth(2021, 1), value),
    "TypeError thrown before cloning PlainYearMonth instance");
  assert.throws(RangeError, () => Temporal.PlainYearMonth.from("1976-11-18Z", value),
    "Invalid string string processed before throwing TypeError");
};
