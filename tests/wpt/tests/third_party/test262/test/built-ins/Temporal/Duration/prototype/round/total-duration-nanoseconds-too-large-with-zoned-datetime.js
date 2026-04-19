// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  NanosecondsToDays throws a RangeError when the number of nanoseconds is too large.
features: [Temporal]
---*/

const maxMs = 9_007_199_254_740_991_487;
const maxUs = 9_007_199_254_740_991_475_711;
const maxNs = 9_007_199_254_740_991_463_129_087;

const durations = [
  Temporal.Duration.from({ seconds: Number.MAX_SAFE_INTEGER }),
  Temporal.Duration.from({ milliseconds: maxMs }),
  Temporal.Duration.from({ microseconds: maxUs }),
  Temporal.Duration.from({ nanoseconds: maxNs }),
  Temporal.Duration.from({ seconds: -Number.MAX_SAFE_INTEGER }),
  Temporal.Duration.from({ milliseconds: -maxMs }),
  Temporal.Duration.from({ microseconds: -maxUs }),
  Temporal.Duration.from({ nanoseconds: -maxNs }),
];

var zonedDateTime = new Temporal.ZonedDateTime(0n, "UTC");

var options = {
  smallestUnit: "day",
  largestUnit: "day",
  relativeTo: zonedDateTime,
};

for (let duration of durations) {
  assert.throws(RangeError, () => duration.round(options));
}
