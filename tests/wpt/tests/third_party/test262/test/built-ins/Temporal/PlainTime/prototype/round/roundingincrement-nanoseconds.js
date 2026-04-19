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
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 1 }),
  3, 34, 56, 987, 654, 321, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 2 }),
  3, 34, 56, 987, 654, 322, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 4 }),
  3, 34, 56, 987, 654, 320, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 5 }),
  3, 34, 56, 987, 654, 320, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 8 }),
  3, 34, 56, 987, 654, 320, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 10 }),
  3, 34, 56, 987, 654, 320, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 20 }),
  3, 34, 56, 987, 654, 320, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 25 }),
  3, 34, 56, 987, 654, 325, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 40 }),
  3, 34, 56, 987, 654, 320, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 50 }),
  3, 34, 56, 987, 654, 300, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 100 }),
  3, 34, 56, 987, 654, 300, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 125 }),
  3, 34, 56, 987, 654, 375, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 200 }),
  3, 34, 56, 987, 654, 400, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 250 }),
  3, 34, 56, 987, 654, 250, "nanoseconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanoseconds", roundingIncrement: 500 }),
  3, 34, 56, 987, 654, 500, "nanoseconds");
