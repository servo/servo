// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Balancing from hours or smaller to weeks or bigger happens correctly.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const day_duration = 100;

const tests = [ ["days", { days: day_duration }],
  ["hours", { hours: day_duration * 24 }],
  ["minutes", { minutes: day_duration * 24 * 60 }],
  ["seconds", { seconds: day_duration * 24 * 60 * 60 }],
  ["milliseconds", { milliseconds: day_duration * 24 * 60 * 60 * 1000 }],
  ["microseconds", { microseconds: day_duration * 24 * 60 * 60 * 1000 * 1000 }],
  ["nanoseconds", { nanoseconds: day_duration * 24 * 60 * 60 * 1000 * 1000 * 1000 }]];

for (const [unit, duration_desc] of tests)
  TemporalHelpers.assertDuration(Temporal.Duration.from(duration_desc).round({ relativeTo: '2023-02-21', largestUnit: 'month' }),
    0, 3, 0, 11, 0, 0, 0, 0, 0, 0, `rounding from ${unit}`);

