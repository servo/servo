// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: >
  BalanceDuration computes floating-point values that are the same as exact math
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Largest temporal unit is "day".
const duration1 = Temporal.Duration.from({seconds: 4503599627370495, nanoseconds: 499_999_999});
const duration2 = Temporal.Duration.from({seconds: 4503599627370495 - 86400, nanoseconds: 499_999_999, days: 1});
const nanos = 4503599627370495_499_999_999n * 2n;

TemporalHelpers.assertDuration(
  duration1.add(duration2),
  0, 0, 0,
  Number((nanos / (24n * 60n * 60n * 1_000_000_000n))),
  Number((nanos / (60n * 60n * 1_000_000_000n)) % 24n),
  Number((nanos / (60n * 1_000_000_000n)) % 60n),
  Number((nanos / 1_000_000_000n) % 60n),
  Number((nanos / 1_000_000n) % 1000n),
  Number((nanos / 1000n) % 1000n),
  Number(nanos % 1000n),
  "duration1.add(duration2)"
);
