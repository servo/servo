// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  Temporal.Duration objects can be passed to format.
features: [Temporal, Intl.DurationFormat]
---*/

var durations = [
  {
    object: new Temporal.Duration(),
    durationLike: {
      years: 0,
    },
  },
  {
    object: new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10),
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

for (var {object, durationLike} of durations) {
  var expected = formatter.format(durationLike);
  var actual = formatter.format(object);
  assert.sameValue(actual, expected, `"${object}"`);
}
