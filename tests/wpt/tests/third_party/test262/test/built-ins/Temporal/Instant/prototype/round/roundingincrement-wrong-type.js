// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Type conversions for roundingIncrement option
info: |
    sec-getoption step 8.a:
      a. Set _value_ to ? ToNumber(value).
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.instant.prototype.round step 13:
      13. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, _maximum_, *false*).
includes: [temporalHelpers.js, compareArray.js]
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_987_654_321n);

TemporalHelpers.checkRoundingIncrementOptionWrongType(
  (roundingIncrement) => instant.round({ smallestUnit: 'second', roundingIncrement }),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_000_001_000_000_000n, descr),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_000_000_000_000_000n, descr),
);
