// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: >
  RoundNumberToIncrementAsIfPositive should multiply the remainder by an extra sign
  before comparing it
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(1938, 4, 24, 22, 13, 20);
const roundedDown = new Temporal.PlainDateTime(1938, 4, 24, 22, 0, 0);
const roundedUp = new Temporal.PlainDateTime(1938, 4, 24, 23, 0, 0);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfCeil" }),
  roundedDown,
  "Rounding with halfCeil rounds to the closest hour"
);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfFloor" }),
  roundedDown,
  "Rounding with halfFloor rounds to the closest hour"
);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfExpand" }),
  roundedDown,
  "Rounding with halfExpand rounds to the closest hour"
);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfTrunc" }),
  roundedDown,
  "Rounding with halfTrunc rounds to the closest hour"
);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfEven" }),
  roundedDown,
  "Rounding with halfEven rounds to the closest hour"
);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "ceil" }),
  roundedUp,
  "Rounding with ceil rounds to the next hour"
);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "floor" }),
  roundedDown,
  "Rounding with floor rounds to the closest hour"
);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "expand" }),
  roundedUp,
  "Rounding with expand rounds to the next hour"
);

TemporalHelpers.assertPlainDateTimesEqual(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "trunc" }),
  roundedDown,
  "Rounding with trunc rounds to the closest hour"
);
