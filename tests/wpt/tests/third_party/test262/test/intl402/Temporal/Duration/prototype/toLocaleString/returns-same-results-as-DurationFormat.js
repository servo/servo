// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tolocalestring
description: >
  Tests that Temporal.Duration.prototype.toLocaleString produces the same
  results as Intl.DurationFormat.
features: [Temporal, Intl.DurationFormat]
---*/

var durationLike = {
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
};

var duration = Temporal.Duration.from(durationLike);

var locales = [
  undefined,
  "en",
  "de",
  "th-u-nu-thai",
  ["ar-u-nu-arab"],
];

var options = [
  undefined,
  {style: "long"},
];

for (var locale of locales) {
  for (var opts of options) {
    var formatter = new Intl.DurationFormat(locale, opts);

    assert.sameValue(
      duration.toLocaleString(locale, opts),
      formatter.format(durationLike),
      `locale="${locale}", options="${JSON.stringify(opts)}", duration="${JSON.stringify(duration)}"`
    );
  }
}
