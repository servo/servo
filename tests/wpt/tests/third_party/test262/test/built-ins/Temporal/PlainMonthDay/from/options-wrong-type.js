// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
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
  assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ monthCode: "M12", day: 15 }, value),
    `TypeError on wrong options type ${typeof value}`);
  assert.throws(TypeError, () => Temporal.PlainMonthDay.from(new Temporal.PlainMonthDay(12, 15), value),
    "TypeError thrown before cloning PlainMonthDay instance");
  assert.throws(RangeError, () => Temporal.PlainMonthDay.from("1976-11-18Z", value),
    "Invalid string string processed before throwing TypeError");
};
