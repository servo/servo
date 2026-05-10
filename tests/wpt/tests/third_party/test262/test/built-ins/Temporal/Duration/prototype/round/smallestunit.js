// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: smallestUnit should be taken into account
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const d = Temporal.Duration.from({
  days: 1,
  hours: 2,
  minutes: 3,
  seconds: 4,
  milliseconds: 5,
  microseconds: 6,
  nanoseconds: 7
});
const tests = {
  'day': [0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
  'hour': [0, 0, 0, 1, 2, 0, 0, 0, 0, 0],
  'minute':  [0, 0, 0, 1, 2, 3, 0, 0, 0, 0],
  'second':  [0, 0, 0, 1, 2, 3, 4, 0, 0, 0],
  'millisecond':  [0, 0, 0, 1, 2, 3, 4, 5, 0, 0],
  'microsecond':  [0, 0, 0, 1, 2, 3, 4, 5, 6, 0],
  'nanosecond':  [0, 0, 0, 1, 2, 3, 4, 5, 6, 7],
};
for (const [smallestUnit, expected] of Object.entries(tests)) {
  TemporalHelpers.assertDuration(d.round(smallestUnit), ...expected,
    `"${smallestUnit}" should work as argument`);
  TemporalHelpers.assertDuration(d.round({ smallestUnit }), ...expected,
    `"${smallestUnit}" should work in option bag`);
}
