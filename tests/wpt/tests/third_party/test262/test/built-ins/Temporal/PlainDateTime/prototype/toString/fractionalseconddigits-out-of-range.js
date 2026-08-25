// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: RangeError thrown when fractionalSecondDigits option out of range
info: |
    sec-getstringornumberoption step 3.a:
      a. If _value_ < _minimum_ or _value_ > _maximum_, throw a *RangeError* exception.
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.plaindatetime.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 650, 0);

assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: -Infinity }),
  "−∞ is out of range for fractionalSecondDigits");
assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: -1 }),
  "−1 is out of range for fractionalSecondDigits");
assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: 10 }),
  "10 is out of range for fractionalSecondDigits");
assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: Infinity }),
  "∞ is out of range for fractionalSecondDigits");
