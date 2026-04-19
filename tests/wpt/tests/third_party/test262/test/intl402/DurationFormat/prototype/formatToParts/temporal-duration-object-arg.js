// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.formatToParts
description: >
  Temporal.Duration objects can be passed to formatToParts.
features: [Temporal, Intl.DurationFormat]
---*/

function assertSameParts(actual, expected) {
  assert.sameValue(actual.length, expected.length);

  for (var i = 0; i < actual.length; ++i) {
    assert.sameValue(actual[i].type, expected[i].type);
    assert.sameValue(actual[i].value, expected[i].value);
    assert.sameValue(actual[i].unit, expected[i].unit);
  }
}

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
  assertSameParts(actual, expected);
}
