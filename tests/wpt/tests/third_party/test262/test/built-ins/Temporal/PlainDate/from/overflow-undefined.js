// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
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
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainDate(2000, 5, 2),
  "2000-05-02",
];
validValues.forEach((value) => {
  const explicit = Temporal.PlainDate.from(value, { overflow: undefined });
  TemporalHelpers.assertPlainDate(explicit, 2000, 5, "M05", 2, "overflow is ignored");
  const implicit = Temporal.PlainDate.from(value, {});
  TemporalHelpers.assertPlainDate(implicit, 2000, 5, "M05", 2, "overflow is ignored");
  const lambda = Temporal.PlainDate.from(value, () => {});
  TemporalHelpers.assertPlainDate(lambda, 2000, 5, "M05", 2, "overflow is ignored");
});

const propertyBag = { year: 2000, month: 13, day: 34 };
const explicit = Temporal.PlainDate.from(propertyBag, { overflow: undefined });
TemporalHelpers.assertPlainDate(explicit, 2000, 12, "M12", 31, "default overflow is constrain");
const implicit = Temporal.PlainDate.from(propertyBag, {});
TemporalHelpers.assertPlainDate(implicit, 2000, 12, "M12", 31, "default overflow is constrain");
const lambda = Temporal.PlainDate.from(propertyBag, () => {});
TemporalHelpers.assertPlainDate(lambda, 2000, 12, "M12", 31, "default overflow is constrain");
