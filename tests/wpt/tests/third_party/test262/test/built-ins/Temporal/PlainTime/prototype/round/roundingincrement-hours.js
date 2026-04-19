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
  plainTime.round({ smallestUnit: "hours", roundingIncrement: 1 }),
  4, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "hours", roundingIncrement: 2 }),
  4, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "hours", roundingIncrement: 3 }),
  3, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "hours", roundingIncrement: 4 }),
  4, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "hours", roundingIncrement: 6 }),
  6, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "hours", roundingIncrement: 8 }),
  0, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "hours", roundingIncrement: 12 }),
  0, 0, 0, 0, 0, 0, "hours");
