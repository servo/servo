// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
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
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainMonthDay(5, 2),
  "05-02",
];
validValues.forEach((value) => {
  const explicit = Temporal.PlainMonthDay.from(value, { overflow: undefined });
  TemporalHelpers.assertPlainMonthDay(explicit, "M05", 2, "overflow is ignored");
  const implicit = Temporal.PlainMonthDay.from(value, {});
  TemporalHelpers.assertPlainMonthDay(implicit, "M05", 2, "overflow is ignored");
  const lambda = Temporal.PlainMonthDay.from(value, () => {});
  TemporalHelpers.assertPlainMonthDay(lambda, "M05", 2, "overflow is ignored");
});

const propertyBag = { year: 2000, month: 13, day: 34 };
const explicit = Temporal.PlainMonthDay.from(propertyBag, { overflow: undefined });
TemporalHelpers.assertPlainMonthDay(explicit, "M12", 31, "default overflow is constrain");
const implicit = Temporal.PlainMonthDay.from(propertyBag, {});
TemporalHelpers.assertPlainMonthDay(implicit, "M12", 31, "default overflow is constrain");
const lambda = Temporal.PlainMonthDay.from(propertyBag, () => {});
TemporalHelpers.assertPlainMonthDay(lambda, "M12", 31, "default overflow is constrain");
