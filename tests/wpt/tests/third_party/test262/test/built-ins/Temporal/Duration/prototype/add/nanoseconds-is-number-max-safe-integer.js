// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: >
  AddDuration computes on exact mathematical number values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Largest temporal unit is "day".
const duration1 = Temporal.Duration.from({nanoseconds: Number.MAX_SAFE_INTEGER});
const duration2 = Temporal.Duration.from({nanoseconds: 2, days: 1});
const nanos = BigInt(Number.MAX_SAFE_INTEGER) + 2n;

TemporalHelpers.assertDuration(
  duration1.add(duration2),
  0, 0, 0,
  1 + Number((nanos / (24n * 60n * 60n * 1_000_000_000n))),
  Number((nanos / (60n * 60n * 1_000_000_000n)) % 24n),
  Number((nanos / (60n * 1_000_000_000n)) % 60n),
  Number((nanos / 1_000_000_000n) % 60n),
  Number((nanos / 1_000_000n) % 1000n),
  Number((nanos / 1000n) % 1000n),
  Number(nanos % 1000n),
  "duration1.add(duration2)"
);
