// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-totemporaldate steps 2–3:
      2. If Type(_item_) is Object, then
        ...
        g. Return ? DateFromFields(_calendar_, _fields_, _options_).
      3. Perform ? ToTemporalOverflow(_options_).
    sec-temporal.plaindate.from steps 2–3:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalDate]] internal slot, then
        a. Perform ? ToTemporalOverflow(_options_).
        b. Return ...
      3. Return ? ToTemporalDate(_item_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainDate(2000, 5, 2),
  { year: 2000, month: 5, day: 2 },
  "2000-05-02",
];
validValues.forEach((value) => TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => Temporal.PlainDate.from(value, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainDate(result, 2000, 5, "M05", 2, descr),
));
