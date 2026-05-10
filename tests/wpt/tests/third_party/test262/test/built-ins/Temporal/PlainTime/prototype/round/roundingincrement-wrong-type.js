// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: Type conversions for roundingIncrement option
info: |
    sec-getoption step 8.a:
      a. Set _value_ to ? ToNumber(value).
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.plaintime.prototype.round step 11:
      11. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, _maximum_, *false*).
includes: [temporalHelpers.js, compareArray.js]
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

TemporalHelpers.checkRoundingIncrementOptionWrongType(
  (roundingIncrement) => time.round({ smallestUnit: 'second', roundingIncrement }),
  (result, descr) => TemporalHelpers.assertPlainTime(result, 12, 34, 57, 0, 0, 0, descr),
  (result, descr) => TemporalHelpers.assertPlainTime(result, 12, 34, 56, 0, 0, 0, descr),
);
