// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Rounding for fractionalSecondDigits option
info: |
    sec-getstringornumberoption step 3.b:
      b. Return floor(ℝ(_value_)).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.duration.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 650, 0);

let string = duration.toString({ fractionalSecondDigits: 2.5 });
assert.sameValue(string, "P1Y2M3W4DT5H6M7.98S", "fractionalSecondDigits 2.5 floors to 2");

string = duration.toString({ fractionalSecondDigits: 9.7 });
assert.sameValue(string, "P1Y2M3W4DT5H6M7.987650000S", "fractionalSecondDigits 9.7 floors to 9 and is not out of range");

assert.throws(
  RangeError,
  () => duration.toString({ fractionalSecondDigits: -0.6 }),
  "fractionalSecondDigits -0.6 floors to -1 and is out of range"
);
