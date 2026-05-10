// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Type conversions for roundingIncrement option
info: |
    sec-getoption step 8.a:
      a. Set _value_ to ? ToNumber(value).
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.plaindate.prototype.since step 12:
      12. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, *undefined*, *false*).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDate(2000, 5, 2);
const later = new Temporal.PlainDate(2000, 5, 7);

TemporalHelpers.checkRoundingIncrementOptionWrongType(
  (roundingIncrement) => later.since(earlier, { roundingIncrement }),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, descr),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, descr),
);
