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
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 1 }),
  0, 0, 0, 0, 10, 35, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 2 }),
  0, 0, 0, 0, 10, 34, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 3 }),
  0, 0, 0, 0, 10, 33, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 4 }),
  0, 0, 0, 0, 10, 32, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 5 }),
  0, 0, 0, 0, 10, 35, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 6 }),
  0, 0, 0, 0, 10, 30, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 10 }),
  0, 0, 0, 0, 10, 30, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 12 }),
  0, 0, 0, 0, 10, 24, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 15 }),
  0, 0, 0, 0, 10, 30, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 20 }),
  0, 0, 0, 0, 10, 20, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingIncrement: 30 }),
  0, 0, 0, 0, 10, 30, 0, 0, 0, 0, "minutes");
