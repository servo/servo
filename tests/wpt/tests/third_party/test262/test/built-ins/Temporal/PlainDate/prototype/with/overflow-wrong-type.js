// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-isodatefromfields step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plaindate.prototype.with step 16:
      16. Return ? DateFromFields(_calendar_, _fields_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);
TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => date.with({ month: 8 }, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainDate(result, 2000, 8, "M08", 2, descr),
);
