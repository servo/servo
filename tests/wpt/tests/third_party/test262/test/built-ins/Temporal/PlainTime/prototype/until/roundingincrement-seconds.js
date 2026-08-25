// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: Valid values for roundingIncrement option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainTime(3, 12, 34, 123, 456, 789);
const later = new Temporal.PlainTime(13, 47, 57, 988, 655, 322);

TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 1 }),
  0, 0, 0, 0, 10, 35, 23, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 2 }),
  0, 0, 0, 0, 10, 35, 22, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 3 }),
  0, 0, 0, 0, 10, 35, 21, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 4 }),
  0, 0, 0, 0, 10, 35, 20, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 5 }),
  0, 0, 0, 0, 10, 35, 20, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 6 }),
  0, 0, 0, 0, 10, 35, 18, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 10 }),
  0, 0, 0, 0, 10, 35, 20, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 12 }),
  0, 0, 0, 0, 10, 35, 12, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 15 }),
  0, 0, 0, 0, 10, 35, 15, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 20 }),
  0, 0, 0, 0, 10, 35, 20, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "seconds", roundingIncrement: 30 }),
  0, 0, 0, 0, 10, 35, 0, 0, 0, 0, "seconds");
