// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Test representative result for all units, without relativeTo
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);

const dayMilliseconds = 24 * 3600 * 1000;
const fullDays = 5;
const fullMilliseconds = fullDays * dayMilliseconds + 5 * 3600_000 + 5 * 60_000 + 5000 + 5;
const partialDayMilliseconds = fullMilliseconds - fullDays * dayMilliseconds + 0.005005;
const fractionalDay = partialDayMilliseconds / dayMilliseconds;
const totalResults = {
  days: fullDays + fractionalDay,
  hours: fullDays * 24 + partialDayMilliseconds / 3600000,
  minutes: fullDays * 24 * 60 + partialDayMilliseconds / 60000,
  seconds: fullDays * 24 * 60 * 60 + partialDayMilliseconds / 1000,
  milliseconds: fullMilliseconds + 0.005005,
  microseconds: fullMilliseconds * 1000 + 5.005,
  nanoseconds: fullMilliseconds * 1000000 + 5005
};
for (const [unit, expected] of Object.entries(totalResults)) {
  assert.sameValue(duration.total(unit), expected, `Duration.total results for ${unit}`);
}
