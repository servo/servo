// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Rounding for fractionalSecondDigits option
info: |
    sec-getstringornumberoption step 3.b:
      b. Return floor(ℝ(_value_)).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.plaindatetime.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 650, 0);

let string = datetime.toString({ fractionalSecondDigits: 2.5 });
assert.sameValue(string, "2000-05-02T12:34:56.98", "fractionalSecondDigits 2.5 floors to 2");

string = datetime.toString({ fractionalSecondDigits: 9.7 });
assert.sameValue(string, "2000-05-02T12:34:56.987650000", "fractionalSecondDigits 9.7 floors to 9 and is not out of range");

assert.throws(
  RangeError,
  () => datetime.toString({ fractionalSecondDigits: -0.6 }),
  "fractionalSecondDigits -0.6 floors to -1 and is out of range"
);
