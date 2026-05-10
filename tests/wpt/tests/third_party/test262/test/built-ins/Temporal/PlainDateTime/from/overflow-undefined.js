// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-interprettemporaldatetimefields steps 2–3:
      2. Let _temporalDate_ be ? DateFromFields(_calendar_, _fields_, _options_).
      3. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal-totemporaldatetime steps 2–3:
      2. If Type(_item_) is Object, then
        ...
        g. Let _result_ be ? InterpretTemporalDateTimeFields(_calendar_, _fields_, _options_).
      3. Else,
        a. Perform ? ToTemporalOverflow(_options_).
    sec-temporal.plaindatetime.from steps 2–3:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalDateTime]] internal slot, then
        a. Perform ? ToTemporalOverflow(_options_).
        b. Return ...
      3. Return ? ToTemporalDateTime(_item_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainDateTime(2000, 5, 2, 12),
  "2000-05-02T12:00",
];
validValues.forEach((value) => {
  const explicit = Temporal.PlainDateTime.from(value, { overflow: undefined });
  TemporalHelpers.assertPlainDateTime(explicit, 2000, 5, "M05", 2, 12, 0, 0, 0, 0, 0, "overflow is ignored");
  const implicit = Temporal.PlainDateTime.from(value, {});
  TemporalHelpers.assertPlainDateTime(implicit, 2000, 5, "M05", 2, 12, 0, 0, 0, 0, 0, "overflow is ignored");
  const lambda = Temporal.PlainDateTime.from(value, () => {});
  TemporalHelpers.assertPlainDateTime(lambda, 2000, 5, "M05", 2, 12, 0, 0, 0, 0, 0, "overflow is ignored");
});

const propertyBag = { year: 2000, month: 13, day: 34, hour: 12 };
const explicit = Temporal.PlainDateTime.from(propertyBag, { overflow: undefined });
TemporalHelpers.assertPlainDateTime(explicit, 2000, 12, "M12", 31, 12, 0, 0, 0, 0, 0, "default overflow is constrain");
const implicit = Temporal.PlainDateTime.from(propertyBag, {});
TemporalHelpers.assertPlainDateTime(implicit, 2000, 12, "M12", 31, 12, 0, 0, 0, 0, 0, "default overflow is constrain");
const lambda = Temporal.PlainDateTime.from(propertyBag, () => {});
TemporalHelpers.assertPlainDateTime(lambda, 2000, 12, "M12", 31, 12, 0, 0, 0, 0, 0, "default overflow is constrain");
