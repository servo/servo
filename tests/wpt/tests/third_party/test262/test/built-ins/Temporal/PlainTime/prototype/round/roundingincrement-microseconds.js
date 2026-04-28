// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: Valid values for roundingIncrement option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(3, 34, 56, 987, 654, 321);

TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 1 }),
  3, 34, 56, 987, 654, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 2 }),
  3, 34, 56, 987, 654, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 4 }),
  3, 34, 56, 987, 656, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 5 }),
  3, 34, 56, 987, 655, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 8 }),
  3, 34, 56, 987, 656, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 10 }),
  3, 34, 56, 987, 650, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 20 }),
  3, 34, 56, 987, 660, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 25 }),
  3, 34, 56, 987, 650, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 40 }),
  3, 34, 56, 987, 640, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 50 }),
  3, 34, 56, 987, 650, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 100 }),
  3, 34, 56, 987, 700, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 125 }),
  3, 34, 56, 987, 625, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 200 }),
  3, 34, 56, 987, 600, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 250 }),
  3, 34, 56, 987, 750, 0, "microseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microseconds", roundingIncrement: 500 }),
  3, 34, 56, 987, 500, 0, "microseconds");
