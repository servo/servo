// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: >
  AddDuration computes on exact mathematical number values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Largest temporal unit is "microsecond".
let duration1 = Temporal.Duration.from({microseconds: Number.MAX_SAFE_INTEGER + 1, nanoseconds: 0});
let duration2 = Temporal.Duration.from({microseconds: 1, nanoseconds: 1000});

TemporalHelpers.assertDuration(
  duration1.add(duration2),
  0, 0, 0, 0,
  0, 0, 0, 0,
  9007199254740994,
  0,
  "duration1.add(duration2)"
);
