// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Round to different smallest increments
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2019, 1, 8, 8, 22, 36, 123, 456, 789);
const later = new Temporal.PlainDateTime(2021, 9, 7, 12, 39, 40, 987, 654, 321);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingIncrement: 3, roundingMode: "halfExpand" }),
  0, 0, 0, 973, 3, 0, 0, 0, 0, 0,
  "rounds to an increment of hours"
);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 30, roundingMode: "halfExpand" }),
  0, 0, 0, 973, 4, 30, 0, 0, 0,0,
  "rounds to an increment of minutes"
);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "seconds", roundingIncrement: 15, roundingMode: "halfExpand" }),
  0, 0, 0, 973, 4, 17, 0, 0, 0, 0,
  "rounds to an increment of seconds"
);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 10, roundingMode: "halfExpand" }),
  0, 0, 0, 973, 4, 17, 4, 860, 0, 0,
  "rounds to an increment of milliseconds"
);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "microseconds", roundingIncrement: 10, roundingMode: "halfExpand" }),
  0, 0, 0, 973, 4, 17, 4, 864, 200, 0,
  "rounds to an increment of microseconds"
);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 10, roundingMode: "halfExpand" }),
  0, 0, 0, 973, 4, 17, 4, 864, 197, 530,
  "rounds to an increment of nanoseconds"
);
