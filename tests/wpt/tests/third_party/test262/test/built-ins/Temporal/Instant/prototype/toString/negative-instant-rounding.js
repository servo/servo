// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: >
  RoundNumberToIncrementAsIfPositive should multiply the remainder by an extra sign
  before comparing it.
features: [Temporal]
---*/

const instance = new Temporal.Instant(-999999999999999990n); // 1938-04-24T22:13:20.00000001Z
const roundedDown = "1938-04-24T22:13:20.000Z";
const roundedUp = "1938-04-24T22:13:20.001Z";

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'halfCeil'}),
  roundedDown,
  "Rounding with halfCeil rounds to the closest millisecond"
);

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'halfFloor'}),
  roundedDown,
  "Rounding with halfFloor rounds to the closest millisecond"
);

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'halfExpand'}),
  roundedDown,
  "Rounding with halfExpand rounds to the closest millisecond"
);

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'halfTrunc'}),
  roundedDown,
  "Rounding with halfTrunc rounds to the closest millisecond"
);

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'halfEven'}),
  roundedDown,
  "Rounding with halfEven rounds to the closest millisecond"
);

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'ceil'}),
  roundedUp,
  "Rounding with ceil rounds to the next millisecond"
);

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'floor'}),
  roundedDown,
  "Rounding with floor rounds to the closest millisecond"
);

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'expand'}),
  roundedUp,
  "Rounding with expand rounds to the next millisecond"
);

assert.sameValue(
  instance.toString({smallestUnit: 'millisecond', roundingMode: 'trunc'}),
  roundedDown,
  "Rounding with trunc rounds to the closest millisecond"
);
