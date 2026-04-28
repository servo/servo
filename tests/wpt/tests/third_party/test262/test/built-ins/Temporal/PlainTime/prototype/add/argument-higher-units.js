// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.add
description: Higher units are ignored.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
const values = [
  new Temporal.Duration(0, 0, 0, 1),
  new Temporal.Duration(0, 0, 1),
  new Temporal.Duration(0, 1),
  new Temporal.Duration(1),
  { days: 1 },
  { weeks: 1 },
  { months: 1 },
  { years: 1 },
  "P1D",
  "P1W",
  "P1M",
  "P1Y",
];
for (const value of values) {
  TemporalHelpers.assertPlainTime(plainTime.add(value),
    15, 23, 30, 123, 456, 789);
}
