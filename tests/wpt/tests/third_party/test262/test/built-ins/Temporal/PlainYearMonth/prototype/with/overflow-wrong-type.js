// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-isoyearmonthfromfields step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plainyearmonth.prototype.with step 16:
      16. Return ? YearMonthFromFields(_calendar_, _fields_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const yearmonth = new Temporal.PlainYearMonth(2000, 5);
TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => yearmonth.with({ month: 8 }, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainYearMonth(result, 2000, 8, "M08", descr),
);
