// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindatetime.prototype.daysinweek
description: Checking days in week for a "normal" case (non-undefined, non-boundary case, etc.)
features: [Temporal]
---*/

const tests = [
  new Temporal.PlainDateTime(1976, 1, 1, 15, 23, 30, 123, 456, 789),
  new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789),
  new Temporal.PlainDateTime(1976, 12, 31, 15, 23, 30, 123, 456, 789),
];
for (const plainDateTime of tests) {
  assert.sameValue(plainDateTime.daysInWeek, 7, `Seven days in the week of ${plainDateTime}`);
}
