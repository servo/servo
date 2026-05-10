// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: RangeError thrown when overflow option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.plaintime.from step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainTime(12),
  { hour: 12 },
  "12:00",
];

const badOverflows = ["", "CONSTRAIN", "balance", "other string", "constra\u0131n", "reject\0"];
for (const value of validValues) {
  for (const overflow of badOverflows) {
    assert.throws(
      RangeError,
      () => Temporal.PlainTime.from(value, { overflow }),
      `invalid overflow ("${overflow}")`
    );
  }
}
