// Copyright (C) 2023 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  The correct separator is used for numeric hours with zero minutes and non-zero seconds.
locale: [en]
includes: [testIntl.js]
features: [Intl.DurationFormat]
---*/

const df = new Intl.DurationFormat("en", {
  // hours must be numeric, so that a time separator is used for the following units.
  hours: "numeric",
});

const durations = [
  // Test all eight possible combinations for zero and non-zero hours, minutes,
  // and seconds.
  {hours: 0, minutes: 0, seconds: 0},
  {hours: 0, minutes: 0, seconds: 1},
  {hours: 0, minutes: 1, seconds: 0},
  {hours: 0, minutes: 1, seconds: 1},
  {hours: 1, minutes: 0, seconds: 0},
  {hours: 1, minutes: 0, seconds: 1},
  {hours: 1, minutes: 1, seconds: 0},
  {hours: 1, minutes: 1, seconds: 1},

  // Additionally test when hours is non-zero and a sub-seconds unit is non-zero,
  // but minutes and seconds are both zero.
  {hours: 1, minutes: 0, seconds: 0, milliseconds: 1},
  {hours: 1, minutes: 0, seconds: 0, microseconds: 1},
  {hours: 1, minutes: 0, seconds: 0, nanoseconds: 1},
];

for (const duration of durations) {
  const expected = formatDurationFormatPattern(df, duration);

  assert.sameValue(
    df.format(duration),
    expected,
    `Duration is ${JSON.stringify(duration)}`
  );
}
