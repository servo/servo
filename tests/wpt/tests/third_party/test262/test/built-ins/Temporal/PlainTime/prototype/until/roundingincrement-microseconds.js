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
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 1 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 2 }),
  0, 0, 0, 0, 10, 35, 23, 865, 198, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 4 }),
  0, 0, 0, 0, 10, 35, 23, 865, 196, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 5 }),
  0, 0, 0, 0, 10, 35, 23, 865, 195, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 8 }),
  0, 0, 0, 0, 10, 35, 23, 865, 192, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 10 }),
  0, 0, 0, 0, 10, 35, 23, 865, 190, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 20 }),
  0, 0, 0, 0, 10, 35, 23, 865, 180, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 25 }),
  0, 0, 0, 0, 10, 35, 23, 865, 175, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 40 }),
  0, 0, 0, 0, 10, 35, 23, 865, 160, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 50 }),
  0, 0, 0, 0, 10, 35, 23, 865, 150, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 100 }),
  0, 0, 0, 0, 10, 35, 23, 865, 100, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 125 }),
  0, 0, 0, 0, 10, 35, 23, 865, 125, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 200 }),
  0, 0, 0, 0, 10, 35, 23, 865, 0, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 250 }),
  0, 0, 0, 0, 10, 35, 23, 865, 0, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.until(later, { smallestUnit: "microseconds", roundingIncrement: 500 }),
  0, 0, 0, 0, 10, 35, 23, 865, 0, 0, "microseconds");
