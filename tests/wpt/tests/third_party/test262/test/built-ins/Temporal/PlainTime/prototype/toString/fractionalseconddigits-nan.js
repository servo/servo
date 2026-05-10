// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: RangeError thrown when fractionalSecondDigits option is NaN
info: |
    sec-getoption step 8.b:
      b. If _value_ is *NaN*, throw a *RangeError* exception.
    sec-getstringornumberoption step 2:
      2. Let _value_ be ? GetOption(_options_, _property_, « Number, String », *undefined*, _fallback_).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.plaintime.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 987, 650, 0);
assert.throws(RangeError, () => time.toString({ fractionalSecondDigits: NaN }));
