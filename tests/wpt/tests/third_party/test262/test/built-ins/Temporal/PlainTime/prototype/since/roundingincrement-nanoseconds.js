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
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 1 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 533, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 2 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 532, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 4 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 532, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 5 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 530, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 8 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 528, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 10 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 530, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 20 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 520, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 25 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 525, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 40 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 520, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 50 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 500, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 100 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 500, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 125 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 500, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 200 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 400, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 250 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 500, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingIncrement: 500 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 500, "nanoseconds");
