// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: RangeError thrown when overflow option not one of the allowed string values
info: |
    sec-getoption step 10:
      10. If _values_ is not *undefined* and _values_ does not contain an element equal to _value_, throw a *RangeError* exception.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-interprettemporaldatetimefields steps 2–3:
      2. Let _temporalDate_ be ? DateFromFields(_calendar_, _fields_, _options_).
      3. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plaindatetime.prototype.with step 16:
      16. Let _result_ be ? InterpretTemporalDateTimeFields(_calendar_, _fields_, _options_).
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12);

const badOverflows = ["", "CONSTRAIN", "balance", "other string", "constra\u0131n", "reject\0"];
for (const overflow of badOverflows) {
  assert.throws(
    RangeError,
    () => datetime.with({ minute: 45 }, { overflow }),
    `invalid overflow ("${overflow}")`
  );
}
