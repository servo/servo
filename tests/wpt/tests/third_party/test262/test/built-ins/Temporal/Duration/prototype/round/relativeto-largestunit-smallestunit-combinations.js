// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test for all combinations of largestUnit and smallestUnit with relativeTo
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const plainRelativeTo = new Temporal.PlainDate(2000, 1, 1);
const zonedRelativeTo = new Temporal.ZonedDateTime(63072000_000_000_000n /* = 1972-01-01T00Z */, "UTC");

const exactResults = {
  years: {
    years: [6],
    months: [5, 6],
    weeks: [5, 6, 1],
    days: [5, 6, 0, 10],
    hours: [5, 6, 0, 10, 5],
    minutes: [5, 6, 0, 10, 5, 5],
    seconds: [5, 6, 0, 10, 5, 5, 5],
    milliseconds: [5, 6, 0, 10, 5, 5, 5, 5],
    microseconds: [5, 6, 0, 10, 5, 5, 5, 5, 5],
    nanoseconds: [5, 6, 0, 10, 5, 5, 5, 5, 5, 5],
  },
  months: {
    months: [0, 66],
    weeks: [0, 66, 1],
    days: [0, 66, 0, 10],
    hours: [0, 66, 0, 10, 5],
    minutes: [0, 66, 0, 10, 5, 5],
    seconds: [0, 66, 0, 10, 5, 5, 5],
    milliseconds: [0, 66, 0, 10, 5, 5, 5, 5],
    microseconds: [0, 66, 0, 10, 5, 5, 5, 5, 5],
    nanoseconds: [0, 66, 0, 10, 5, 5, 5, 5, 5, 5],
  },
  weeks: {
    weeks: [0, 0, 288],
    days: [0, 0, 288, 2],
    hours: [0, 0, 288, 2, 5],
    minutes: [0, 0, 288, 2, 5, 5],
    seconds: [0, 0, 288, 2, 5, 5, 5],
    milliseconds: [0, 0, 288, 2, 5, 5, 5, 5],
    microseconds: [0, 0, 288, 2, 5, 5, 5, 5, 5],
    nanoseconds: [0, 0, 288, 2, 5, 5, 5, 5, 5, 5],
  },
  days: {
    days: [0, 0, 0, 2018],
    hours: [0, 0, 0, 2018, 5],
    minutes: [0, 0, 0, 2018, 5, 5],
    seconds: [0, 0, 0, 2018, 5, 5, 5],
    milliseconds: [0, 0, 0, 2018, 5, 5, 5, 5],
    microseconds: [0, 0, 0, 2018, 5, 5, 5, 5, 5],
    nanoseconds: [0, 0, 0, 2018, 5, 5, 5, 5, 5, 5],
  },
  hours: {
    hours: [0, 0, 0, 0, 48437],
    minutes: [0, 0, 0, 0, 48437, 5],
    seconds: [0, 0, 0, 0, 48437, 5, 5],
    milliseconds: [0, 0, 0, 0, 48437, 5, 5, 5],
    microseconds: [0, 0, 0, 0, 48437, 5, 5, 5, 5],
    nanoseconds: [0, 0, 0, 0, 48437, 5, 5, 5, 5, 5],
  },
  minutes: {
    minutes: [0, 0, 0, 0, 0, 2906225],
    seconds: [0, 0, 0, 0, 0, 2906225, 5],
    milliseconds: [0, 0, 0, 0, 0, 2906225, 5, 5],
    microseconds: [0, 0, 0, 0, 0, 2906225, 5, 5, 5],
    nanoseconds: [0, 0, 0, 0, 0, 2906225, 5, 5, 5, 5],
  },
  seconds: {
    seconds: [0, 0, 0, 0, 0, 0, 174373505],
    milliseconds: [0, 0, 0, 0, 0, 0, 174373505, 5],
    microseconds: [0, 0, 0, 0, 0, 0, 174373505, 5, 5],
    nanoseconds: [0, 0, 0, 0, 0, 0, 174373505, 5, 5, 5],
  },
  milliseconds: {
    milliseconds: [0, 0, 0, 0, 0, 0, 0, 174373505005],
    microseconds: [0, 0, 0, 0, 0, 0, 0, 174373505005, 5],
    nanoseconds: [0, 0, 0, 0, 0, 0, 0, 174373505005, 5, 5],
  },
  microseconds: {
    microseconds: [0, 0, 0, 0, 0, 0, 0, 0, 174373505005005],
    nanoseconds: [0, 0, 0, 0, 0, 0, 0, 0, 174373505005005, 5],
  },
};
for (const [largestUnit, entry] of Object.entries(exactResults)) {
  for (const [smallestUnit, expected] of Object.entries(entry)) {
    for (const relativeTo of [plainRelativeTo, zonedRelativeTo]) {
      const [y, mon = 0, w = 0, d = 0, h = 0, min = 0, s = 0, ms = 0, µs = 0, ns = 0] = expected;
      TemporalHelpers.assertDuration(
        duration.round({ largestUnit, smallestUnit, relativeTo }),
        y, mon, w, d, h, min, s, ms, µs, ns,
        `Combination of largestUnit ${largestUnit} and smallestUnit ${smallestUnit}, relative to ${relativeTo}`
      );
    }
  }
}

// 174373505005005005 is not a safe integer.
// ℝ(𝔽(174373505005005005)) == 174373505005004992

for (const relativeTo of [plainRelativeTo, zonedRelativeTo]) {
  TemporalHelpers.assertDuration(
    duration.round({ largestUnit: "nanoseconds", smallestUnit: "nanoseconds", relativeTo }),
    0, 0, 0, 0, 0, 0, 0, 0, 0, 174373505005004992,
    `Combination of largestUnit nanoseconds and smallestUnit nanoseconds, with precision loss, relative to ${relativeTo}`
  );
}
