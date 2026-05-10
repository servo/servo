// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test for all combinations of largestUnit and smallestUnit without relativeTo
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 5, 5, 5, 5, 5, 5, 5);

const exactResults = {
  days: {
    days: [5],
    hours: [5, 5],
    minutes: [5, 5, 5],
    seconds: [5, 5, 5, 5],
    milliseconds: [5, 5, 5, 5, 5],
    microseconds: [5, 5, 5, 5, 5, 5],
    nanoseconds: [5, 5, 5, 5, 5, 5, 5],
  },
  hours: {
    hours: [0, 125],
    minutes: [0, 125, 5],
    seconds: [0, 125, 5, 5],
    milliseconds: [0, 125, 5, 5, 5],
    microseconds: [0, 125, 5, 5, 5, 5],
    nanoseconds: [0, 125, 5, 5, 5, 5, 5],
  },
  minutes: {
    minutes: [0, 0, 7505],
    seconds: [0, 0, 7505, 5],
    milliseconds: [0, 0, 7505, 5, 5],
    microseconds: [0, 0, 7505, 5, 5, 5],
    nanoseconds: [0, 0, 7505, 5, 5, 5, 5],
  },
  seconds: {
    seconds: [0, 0, 0, 450305],
    milliseconds: [0, 0, 0, 450305, 5],
    microseconds: [0, 0, 0, 450305, 5, 5],
    nanoseconds: [0, 0, 0, 450305, 5, 5, 5],
  },
  milliseconds: {
    milliseconds: [0, 0, 0, 0, 450305005],
    microseconds: [0, 0, 0, 0, 450305005, 5],
    nanoseconds: [0, 0, 0, 0, 450305005, 5, 5],
  },
  microseconds: {
    microseconds: [0, 0, 0, 0, 0, 450305005005],
    nanoseconds: [0, 0, 0, 0, 0, 450305005005, 5],
  },
  nanoseconds: {
    nanoseconds: [0, 0, 0, 0, 0, 0, 450305005005005],
  },
};
for (const [largestUnit, entry] of Object.entries(exactResults)) {
  for (const [smallestUnit, expected] of Object.entries(entry)) {
    const [d = 0, h = 0, min = 0, s = 0, ms = 0, µs = 0, ns = 0] = expected;
    TemporalHelpers.assertDuration(
      duration.round({ largestUnit, smallestUnit }),
      0, 0, 0, d, h, min, s, ms, µs, ns,
      `Combination of largestUnit ${largestUnit} and smallestUnit ${smallestUnit}`
    );
  }
}
