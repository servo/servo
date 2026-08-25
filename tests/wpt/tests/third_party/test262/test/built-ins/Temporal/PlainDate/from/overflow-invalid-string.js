// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: RangeError thrown when overflow option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
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
features: [Temporal]
---*/

const validItems = [
  new Temporal.PlainDate(2000, 5, 2),
  new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC"),
  new Temporal.PlainDateTime(2000, 5, 2, 12),
  { year: 2000, month: 5, day: 2 },
  "2000-05-02",
];

const badOverflows = ["", "CONSTRAIN", "balance", "other string", "constra\u0131n", "reject\0"];
for (const item of validItems) {
  for (const overflow of badOverflows) {
    assert.throws(
      RangeError,
      () => Temporal.PlainDate.from(item, { overflow }),
      `invalid overflow ("${overflow}")`
    );
  }
}
