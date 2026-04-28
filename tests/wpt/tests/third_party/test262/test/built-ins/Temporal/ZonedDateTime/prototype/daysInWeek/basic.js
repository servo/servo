// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.daysinweek
description: Checking days in week for a "normal" case (non-undefined, non-boundary case, etc.)
features: [Temporal]
---*/

const tests = [
  new Temporal.ZonedDateTime(189357810123456789n, "UTC"),
  new Temporal.ZonedDateTime(217178610123456789n, "UTC"),
  new Temporal.ZonedDateTime(220893810123456789n, "UTC"),
];
for (const zonedDateTime of tests) {
  assert.sameValue(zonedDateTime.daysInWeek, 7, `Seven days in the week of ${zonedDateTime}`);
}
