// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.with
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.plaintime.prototype.with step 11:
      11. Let _overflow_ be ? ToTemporalOverflow(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12);
TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => time.with({ minute: 45 }, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainTime(result, 12, 45, 0, 0, 0, 0, descr),
);
