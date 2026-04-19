// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
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
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainYearMonth(2000, 5),
  "2000-05",
];
validValues.forEach((value) => {
  const explicit = Temporal.PlainYearMonth.from(value, { overflow: undefined });
  TemporalHelpers.assertPlainYearMonth(explicit, 2000, 5, "M05", "overflow is ignored");
  const implicit = Temporal.PlainYearMonth.from(value, {});
  TemporalHelpers.assertPlainYearMonth(implicit, 2000, 5, "M05", "overflow is ignored");
  const lambda = Temporal.PlainYearMonth.from(value, () => {});
  TemporalHelpers.assertPlainYearMonth(lambda, 2000, 5, "M05", "overflow is ignored");
});

const propertyBag = { year: 2000, month: 13 };
const explicit = Temporal.PlainYearMonth.from(propertyBag, { overflow: undefined });
TemporalHelpers.assertPlainYearMonth(explicit, 2000, 12, "M12", "default overflow is constrain");
const implicit = Temporal.PlainYearMonth.from(propertyBag, {});
TemporalHelpers.assertPlainYearMonth(implicit, 2000, 12, "M12", "default overflow is constrain");
const lambda = Temporal.PlainYearMonth.from(propertyBag, () => {});
TemporalHelpers.assertPlainYearMonth(lambda, 2000, 12, "M12", "default overflow is constrain");
