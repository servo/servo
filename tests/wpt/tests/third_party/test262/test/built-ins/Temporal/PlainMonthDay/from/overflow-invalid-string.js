// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: RangeError thrown when overflow option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
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
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainMonthDay(5, 2),
  { monthCode: "M05", day: 2 },
  "05-02",
];

const badOverflows = ["", "CONSTRAIN", "balance", "other string", "constra\u0131n", "reject\0"];
for (const value of validValues) {
  for (const overflow of badOverflows) {
    assert.throws(
      RangeError,
      () => Temporal.PlainMonthDay.from(value, { overflow }),
      `invalid overflow ("${overflow}")`
    );
  }
}
