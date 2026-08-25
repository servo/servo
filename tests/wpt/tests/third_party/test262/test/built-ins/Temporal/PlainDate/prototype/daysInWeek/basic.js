// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.daysinweek
description: Basic tests for daysInWeek().
features: [Temporal]
---*/

const tests = [
  new Temporal.PlainDate(1976, 1, 1),
  new Temporal.PlainDate(1976, 11, 18),
  new Temporal.PlainDate(1976, 12, 31),
];
for (const plainDate of tests) {
  assert.sameValue(plainDate.daysInWeek, 7, `Seven days in the week of ${plainDate}`);
}
