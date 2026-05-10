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
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 1 }),
  0, 0, 0, 0, 10, 35, 23, 865, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 2 }),
  0, 0, 0, 0, 10, 35, 23, 864, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 4 }),
  0, 0, 0, 0, 10, 35, 23, 864, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 5 }),
  0, 0, 0, 0, 10, 35, 23, 865, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 8 }),
  0, 0, 0, 0, 10, 35, 23, 864, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 10 }),
  0, 0, 0, 0, 10, 35, 23, 860, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 20 }),
  0, 0, 0, 0, 10, 35, 23, 860, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 25 }),
  0, 0, 0, 0, 10, 35, 23, 850, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 40 }),
  0, 0, 0, 0, 10, 35, 23, 840, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 50 }),
  0, 0, 0, 0, 10, 35, 23, 850, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 100 }),
  0, 0, 0, 0, 10, 35, 23, 800, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 125 }),
  0, 0, 0, 0, 10, 35, 23, 750, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 200 }),
  0, 0, 0, 0, 10, 35, 23, 800, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 250 }),
  0, 0, 0, 0, 10, 35, 23, 750, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingIncrement: 500 }),
  0, 0, 0, 0, 10, 35, 23, 500, 0, 0, "milliseconds");
