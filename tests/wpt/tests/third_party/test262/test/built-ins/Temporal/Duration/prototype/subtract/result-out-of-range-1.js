// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: >
  BalanceDuration throws a RangeError when the result is too large.
features: [Temporal]
---*/

const maxSec = Number.MAX_SAFE_INTEGER;
const maxMs = 9_007_199_254_740_991_487;
const maxUs = 9_007_199_254_740_991_475_711;
const maxNs = 9_007_199_254_740_991_463_129_087;

const durations = [
  Temporal.Duration.from({seconds: maxSec}),
  Temporal.Duration.from({milliseconds: maxMs}),
  Temporal.Duration.from({microseconds: maxUs}),
  Temporal.Duration.from({nanoseconds: maxNs}),
  Temporal.Duration.from({seconds: -maxSec}),
  Temporal.Duration.from({milliseconds: -maxMs}),
  Temporal.Duration.from({microseconds: -maxUs}),
  Temporal.Duration.from({nanoseconds: -maxNs}),
];

for (let duration of durations) {
  assert.throws(RangeError, () => {
    duration.subtract(duration.negated());
  }, `subtracting the negation of a large duration from the duration is out of bounds: ${duration}`);
}
