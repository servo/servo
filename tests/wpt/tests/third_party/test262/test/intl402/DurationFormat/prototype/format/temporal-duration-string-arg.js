// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  Temporal duration strings can be passed to format.
features: [Temporal, Intl.DurationFormat]
---*/

var durations = [
  {
    string: "PT0S",
    durationLike: {
      years: 0,
    },
  },
  {
    string: "P1Y2M3W4DT5H6M7.00800901S",
    durationLike: {
      years: 1,
      months: 2,
      weeks: 3,
      days: 4,
      hours: 5,
      minutes: 6,
      seconds: 7,
      milliseconds: 8,
      microseconds: 9,
      nanoseconds: 10,
    },
  },
];

var formatter = new Intl.DurationFormat();

for (var {string, durationLike} of durations) {
  var expected = formatter.format(durationLike);
  var actual = formatter.format(string);
  assert.sameValue(actual, expected, `"${string}"`);
}
