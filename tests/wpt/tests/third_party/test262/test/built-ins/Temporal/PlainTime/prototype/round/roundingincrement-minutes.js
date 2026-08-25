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
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 1 }),
  3, 35, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 2 }),
  3, 34, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 3 }),
  3, 36, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 4 }),
  3, 36, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 5 }),
  3, 35, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 6 }),
  3, 36, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 10 }),
  3, 30, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 12 }),
  3, 36, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 15 }),
  3, 30, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 20 }),
  3, 40, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minutes", roundingIncrement: 30 }),
  3, 30, 0, 0, 0, 0, "minutes");
