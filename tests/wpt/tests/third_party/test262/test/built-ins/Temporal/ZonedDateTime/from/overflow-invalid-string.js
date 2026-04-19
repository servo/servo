// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: RangeError thrown when overflow option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-interprettemporaldatetimefields steps 2–3:
      2. Let _temporalDate_ be ? DateFromFields(_calendar_, _fields_, _options_).
      3. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal-totemporalzoneddatetime steps 2–3:
      2. If Type(_item_) is Object, then
        ...
        j. Let _result_ be ? InterpretTemporalDateTimeFields(_calendar_, _fields_, _options_).
      3. Else,
        a. Perform ? ToTemporalOverflow(_options_).
    sec-temporal.zoneddatetime.from steps 2–3:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. Perform ? ToTemporalOverflow(_options_).
        ...
        d. Return ...
      3. Return ? ToTemporalZonedDateTime(_item_, _options_).
features: [Temporal]
---*/

const validValues = [
  new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC"),
  { year: 2000, month: 5, day: 2, hour: 12, timeZone: "UTC" },
  "2001-09-09T01:46:40.987654321+00:00[UTC]",
];

const badOverflows = ["", "CONSTRAIN", "balance", "other string", "constra\u0131n", "reject\0"];
for (const value of validValues) {
  for (const overflow of badOverflows) {
    assert.throws(
      RangeError,
      () => Temporal.ZonedDateTime.from(value, { overflow }),
      `invalid overflow ("${overflow}")`
    );
  }
}
