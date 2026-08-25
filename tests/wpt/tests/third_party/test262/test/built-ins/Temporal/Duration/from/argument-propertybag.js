// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: Basic propertybag arguments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertDuration(Temporal.Duration.from({
  years: 0,
  months: 0,
  weeks: 0,
  days: 0,
  hours: 0,
  minutes: 0,
  seconds: 0,
  milliseconds: 0,
  microseconds: 0,
  nanoseconds: 0
}), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(Temporal.Duration.from({
  years: 1,
  months: 2,
  weeks: 3,
  days: 4,
  hours: 5,
  minutes: 6,
  seconds: 7,
  milliseconds: 8,
  microseconds: 9,
  nanoseconds: 10
}), 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);

TemporalHelpers.assertDuration(Temporal.Duration.from({
  years: -1,
  months: -2,
  weeks: -3,
  days: -4,
  hours: -5,
  minutes: -6,
  seconds: -7,
  milliseconds: -8,
  microseconds: -9,
  nanoseconds: -10
}), -1, -2, -3, -4, -5, -6, -7, -8, -9, -10);
