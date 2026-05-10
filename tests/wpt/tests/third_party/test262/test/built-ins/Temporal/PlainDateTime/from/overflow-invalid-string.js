// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: RangeError thrown when overflow option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
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
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainDateTime(2000, 5, 2, 12),
  new Temporal.PlainDate(2000, 5, 2),
  new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC"),
  { year: 2000, month: 5, day: 2, hour: 12 },
  "2000-05-02T12:00",
];

const badOverflows = ["", "CONSTRAIN", "balance", "other string", "constra\u0131n", "reject\0"];
for (const value of validValues) {
  for (const overflow of badOverflows) {
    assert.throws(
      RangeError,
      () => Temporal.PlainDateTime.from(value, { overflow }),
      `invalid overflow ("${overflow}")`
    );
  }
}
