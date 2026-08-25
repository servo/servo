// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: Basic behavior
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration1 = Temporal.Duration.from({ days: 1, minutes: 5 });
TemporalHelpers.assertDuration(duration1.add({ days: 2, minutes: 5 }),
  0, 0, 0, 3, 0, 10, 0, 0, 0, 0, "positive same units");
TemporalHelpers.assertDuration(duration1.add({ hours: 12, seconds: 30 }),
  0, 0, 0, 1, 12, 5, 30, 0, 0, 0, "positive different units");
TemporalHelpers.assertDuration(Temporal.Duration.from("P3DT10M").add({ days: -2, minutes: -5 }),
  0, 0, 0, 1, 0, 5, 0, 0, 0, 0, "negative same units");
TemporalHelpers.assertDuration(Temporal.Duration.from("P1DT12H5M30S").add({ hours: -12, seconds: -30 }),
  0, 0, 0, 1, 0, 5, 0, 0, 0, 0, "negative different units");
const duration2 = Temporal.Duration.from("P50DT50H50M50.500500500S");
TemporalHelpers.assertDuration(duration2.add(duration2),
  0, 0, 0, 104, 5, 41, 41, 1, 1, 0, "balancing positive");
const duration3 = Temporal.Duration.from({ hours: -1, seconds: -60 });
TemporalHelpers.assertDuration(duration3.add({ minutes: 122 }),
  0, 0, 0, 0, 1, 1, 0, 0, 0, 0, "balancing flipped sign 1");
const duration4 = Temporal.Duration.from({ hours: -1, seconds: -3721 });
TemporalHelpers.assertDuration(duration4.add({ minutes: 61, nanoseconds: 3722000000001 }),
  0, 0, 0, 0, 0, 1, 1, 0, 0, 1, "balancing flipped sign 2");
TemporalHelpers.assertDuration(duration1.add({ month: 1, days: 1 }),
  0, 0, 0, 2, 0, 5, 0, 0, 0, 0,
  "incorrectly-spelled properties are ignored");
