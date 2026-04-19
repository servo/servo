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
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 1 }),
  3, 34, 57, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 2 }),
  3, 34, 56, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 3 }),
  3, 34, 57, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 4 }),
  3, 34, 56, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 5 }),
  3, 34, 55, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 6 }),
  3, 34, 54, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 10 }),
  3, 35, 0, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 12 }),
  3, 35, 0, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 15 }),
  3, 35, 0, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 20 }),
  3, 35, 0, 0, 0, 0, "seconds");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "seconds", roundingIncrement: 30 }),
  3, 35, 0, 0, 0, 0, "seconds");
