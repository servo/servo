// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-totemporalyearmonth steps 2–3:
      2. If Type(_item_) is Object, then
        ...
        e. Return ? YearMonthFromFields(_calendar_, _fields_, _options_).
      3. Perform ? ToTemporalOverflow(_options_).
    sec-temporal.plainyearmonth.from steps 2–3:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalYearMonth]] internal slot, then
        a. Perform ? ToTemporalOverflow(_options_).
        b. Return ...
      3. Return ? ToTemporalYearMonth(_item_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainYearMonth(2000, 5),
  { year: 2000, month: 5 },
  "2000-05",
];
validValues.forEach((value) => TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => Temporal.PlainYearMonth.from(value, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainYearMonth(result, 2000, 5, "M05", descr),
));
