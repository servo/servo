// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: RangeError thrown when overflow option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-isodatefromfields step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plaindate.prototype.with step 16:
      16. Return ? DateFromFields(_calendar_, _fields_, _options_).
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);

const badOverflows = ["", "CONSTRAIN", "balance", "other string", "constra\u0131n", "reject\0"];
for (const overflow of badOverflows) {
  assert.throws(
    RangeError,
    () => date.with({ month: 8 }, { overflow }),
    `invalid overflow ("${overflow}")`
  );
}
