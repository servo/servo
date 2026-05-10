// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.plaintime.from step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainTime(12),
  { hour: 12 },
  "12:00",
];
validValues.forEach((value) => TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => Temporal.PlainTime.from(value, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainTime(result, 12, 0, 0, 0, 0, 0, descr),
));
