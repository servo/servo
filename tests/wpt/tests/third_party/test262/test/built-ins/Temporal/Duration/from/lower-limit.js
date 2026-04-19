// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: Minimum value is zero.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const units = [
  "years",
  "months",
  "weeks",
  "days",
  "hours",
  "minutes",
  "seconds",
  "milliseconds",
  "microseconds",
  "nanoseconds"
];

units.forEach(unit => TemporalHelpers.assertDuration(Temporal.Duration.from({ [unit]: 0 }),
                                                     0, 0, 0, 0, 0, 0, 0, 0, 0, 0));

[
  "P0Y",
  "P0M",
  "P0W",
  "P0D",
  "PT0H",
  "PT0M",
  "PT0S"
].forEach(str => TemporalHelpers.assertDuration(Temporal.Duration.from(str),
                                                0, 0, 0, 0, 0, 0, 0, 0, 0, 0));
