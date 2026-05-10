// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: Basic behavior
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = Temporal.Duration.from({ days: 3, hours: 1, minutes: 10 });
TemporalHelpers.assertDuration(duration.subtract({ days: 1, minutes: 5 }),
  0, 0, 0, 2, 1, 5, 0, 0, 0, 0);
TemporalHelpers.assertDuration(duration.subtract(duration),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(duration.subtract({ days: 3 }),
  0, 0, 0, 0, 1, 10, 0, 0, 0, 0);
TemporalHelpers.assertDuration(duration.subtract({ minutes: 10 }),
  0, 0, 0, 3, 1, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(duration.subtract({ minutes: 15 }),
  0, 0, 0, 3, 0, 55, 0, 0, 0, 0);
TemporalHelpers.assertDuration(duration.subtract({ seconds: 30 }),
  0, 0, 0, 3, 1, 9, 30, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from('P2DT1H5M').subtract({ days: -1, minutes: -5 }),
  0, 0, 0, 3, 1, 10, 0, 0, 0, 0);
TemporalHelpers.assertDuration(new Temporal.Duration().subtract({ days: -3, hours: -1, minutes: -10 }),
  0, 0, 0, 3, 1, 10, 0, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from('PT1H10M').subtract({ days: -3 }),
  0, 0, 0, 3, 1, 10, 0, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from('P3DT1H').subtract({ minutes: -10 }),
  0, 0, 0, 3, 1, 10, 0, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from('P3DT55M').subtract({ minutes: -15 }),
  0, 0, 0, 3, 1, 10, 0, 0, 0, 0);
TemporalHelpers.assertDuration(Temporal.Duration.from('P3DT1H9M30S').subtract({ seconds: -30 }),
  0, 0, 0, 3, 1, 10, 0, 0, 0, 0);
const d = Temporal.Duration.from({
  minutes: 100,
  seconds: 100,
  milliseconds: 2000,
  microseconds: 2000,
  nanoseconds: 2000
});
const less = Temporal.Duration.from({
  minutes: 10,
  seconds: 10,
  milliseconds: 500,
  microseconds: 500,
  nanoseconds: 500
});
TemporalHelpers.assertDuration(d.subtract(less),
  0, 0, 0, 0, 0, 91, 31, 501, 501, 500);
const tenDays = Temporal.Duration.from('P10D');
const tenMinutes = Temporal.Duration.from('PT10M');
TemporalHelpers.assertDuration(tenDays.subtract({ days: 15 }),
  0, 0, 0, -5, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(tenMinutes.subtract({ minutes: 15 }),
  0, 0, 0, 0, 0, -5, 0, 0, 0, 0);
const d1 = Temporal.Duration.from({ hours: 1, seconds: 60 });
TemporalHelpers.assertDuration(d1.subtract({ minutes: 122 }),
  0, 0, 0, 0, -1, -1, 0, 0, 0, 0);
const d2 = Temporal.Duration.from({ hours: 1, seconds: 3721 });
TemporalHelpers.assertDuration(d2.subtract({ minutes: 61, nanoseconds: 3722000000001 }),
  0, 0, 0, 0, 0, -1, -1, 0, 0, -1);
TemporalHelpers.assertDuration(duration.subtract({ month: 1, days: 1 }),
  0, 0, 0, 2, 1, 10, 0, 0, 0, 0,
  "incorrectly-spelled properties are ignored");
