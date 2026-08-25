// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: Valid values for roundingIncrement option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainTime(3, 12, 34, 123, 456, 789);
const later = new Temporal.PlainTime(13, 47, 57, 988, 655, 322);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingIncrement: 1 }),
  0, 0, 0, 0, 10, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingIncrement: 2 }),
  0, 0, 0, 0, 10, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingIncrement: 3 }),
  0, 0, 0, 0, 9, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingIncrement: 4 }),
  0, 0, 0, 0, 8, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingIncrement: 6 }),
  0, 0, 0, 0, 6, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingIncrement: 8 }),
  0, 0, 0, 0, 8, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingIncrement: 12 }),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "hours");
