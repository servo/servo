// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: >
  RoundNumberToIncrementAsIfPositive should multiply the remainder by an extra sign
  before comparing it
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(-1000000000000000000n, "UTC");  // 1938-04-24T22:13:20Z
const roundedDown = -1000000800000000000n; // 1938-04-24T22:00:00Z
const roundedUp = -999997200000000000n; // 1938-04-24T23:00:00Z

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfCeil" }).epochNanoseconds,
  roundedDown,
  "Rounding with halfCeil rounds to the closest hour"
);

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfFloor" }).epochNanoseconds,
  roundedDown,
  "Rounding with halfFloor rounds to the closest hour"
);

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfExpand" }).epochNanoseconds,
  roundedDown,
  "Rounding with halfExpand rounds to the closest hour"
);

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfTrunc" }).epochNanoseconds,
  roundedDown,
  "Rounding with halfTrunc rounds to the closest hour"
);

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "halfEven" }).epochNanoseconds,
  roundedDown,
  "Rounding with halfEven rounds to the closest hour"
);

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "ceil" }).epochNanoseconds,
  roundedUp,
  "Rounding with ceil rounds to the next hour"
);

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "floor" }).epochNanoseconds,
  roundedDown,
  "Rounding with floor rounds to the closest hour"
);

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "expand" }).epochNanoseconds,
  roundedUp,
  "Rounding with expand rounds to the next hour"
);

assert.sameValue(
  instance.round({ smallestUnit: "hour", roundingIncrement: 1, roundingMode: "trunc" }).epochNanoseconds,
  roundedDown,
  "Rounding with trunc rounds to the closest hour"
);
