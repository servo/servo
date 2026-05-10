// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: RangeError thrown when fractionalSecondDigits option is NaN
info: |
    sec-getoption step 8.b:
      b. If _value_ is *NaN*, throw a *RangeError* exception.
    sec-getstringornumberoption step 2:
      2. Let _value_ be ? GetOption(_options_, _property_, « Number, String », *undefined*, _fallback_).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.instant.prototype.tostring step 6:
      6. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_987_650_000n);
assert.throws(RangeError, () => instant.toString({ fractionalSecondDigits: NaN }));
