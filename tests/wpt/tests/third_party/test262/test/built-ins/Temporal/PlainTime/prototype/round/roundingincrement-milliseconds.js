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
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 1 }),
  3, 34, 56, 988, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 2 }),
  3, 34, 56, 988, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 4 }),
  3, 34, 56, 988, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 5 }),
  3, 34, 56, 990, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 8 }),
  3, 34, 56, 984, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 10 }),
  3, 34, 56, 990, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 20 }),
  3, 34, 56, 980, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 25 }),
  3, 34, 57, 0, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 40 }),
  3, 34, 57, 0, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 50 }),
  3, 34, 57, 0, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 100 }),
  3, 34, 57, 0, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 125 }),
  3, 34, 57, 0, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 200 }),
  3, 34, 57, 0, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 250 }),
  3, 34, 57, 0, 0, 0, "milliseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "milliseconds", roundingIncrement: 500 }),
  3, 34, 57, 0, 0, 0, "milliseconds");
