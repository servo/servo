// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: >
  Fractional parts are computed using exact mathematical values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const tests = {
  "PT0.999999999H": Temporal.Duration.from({
    minutes: 59,
    seconds: 59,
    milliseconds: 999,
    microseconds: 996,
    nanoseconds: 400,
  }),
  "PT0.000000011H": Temporal.Duration.from({
    minutes: 0,
    seconds: 0,
    milliseconds: 0,
    microseconds: 39,
    nanoseconds: 600,
  }),

  "PT0.999999999M": Temporal.Duration.from({
    seconds: 59,
    milliseconds: 999,
    microseconds: 999,
    nanoseconds: 940,
  }),
  "PT0.000000011M": Temporal.Duration.from({
    seconds: 0,
    milliseconds: 0,
    microseconds: 0,
    nanoseconds: 660,
  }),

  "PT0.999999999S": Temporal.Duration.from({
    milliseconds: 999,
    microseconds: 999,
    nanoseconds: 999,
  }),
  "PT0.000000011S": Temporal.Duration.from({
    milliseconds: 0,
    microseconds: 0,
    nanoseconds: 11,
  }),
};

for (let [str, expected] of Object.entries(tests)) {
  let actual = Temporal.Duration.from(str);
  TemporalHelpers.assertDurationsEqual(actual, expected, str);
}
