// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Using lower units in add() throws
features: [Temporal]
---*/

const ym = Temporal.PlainYearMonth.from("2019-11");

const tests = [
  { days: 1 },
  { days: 29 },
  { hours: 1 },
  { minutes: 1 },
  { seconds: 1 },
  { milliseconds: 1 },
  { microseconds: 1 },
  { nanoseconds: 1 },
  { days: 30 },
  { days: 31 },
  { days: 60 },
  { days: 61 },
  { hours: 720 },
  { minutes: 43200 },
  { seconds: 2592000 },
  { milliseconds: 2592000_000 },
  { microseconds: 2592000_000_000 },
  { nanoseconds: 2592000_000_000_000 },
];

for (const argument of tests) {
  assert.throws(RangeError, function () { ym.add(argument); }, "adding a unit lower than months should throw, no options");
  assert.throws(RangeError, function () { ym.add(argument, { overflow: "constrain" }); }, "adding a unit lower than months should throw, constrain");
  assert.throws(RangeError, function () { ym.add(argument, { overflow: "reject" }); }, "adding a unit lower than months should throw, reject");
}
