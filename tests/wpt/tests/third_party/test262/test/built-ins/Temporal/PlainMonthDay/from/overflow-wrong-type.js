// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-totemporalmonthday steps 3–4:
      3. If Type(_item_) is Object, then
        ...
        j. Return ? MonthDayFromFields(_calendar_, _fields_, _options_).
      4. Perform ? ToTemporalOverflow(_options_).
    sec-temporal.plainmonthday.from steps 2–3:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalMonthDay]] internal slot, then
        a. Perform ? ToTemporalOverflow(_options_).
        b. Return ...
      3. Return ? ToTemporalMonthDay(_item_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainMonthDay(5, 2),
  { monthCode: "M05", day: 2 },
  "05-02",
];
validValues.forEach((value) => TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => Temporal.PlainMonthDay.from(value, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainMonthDay(result, "M05", 2, descr),
));
