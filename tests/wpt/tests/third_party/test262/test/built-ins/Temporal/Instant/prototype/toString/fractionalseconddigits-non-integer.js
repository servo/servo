// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Rounding for fractionalSecondDigits option
info: |
    sec-getstringornumberoption step 3.b:
      b. Return floor(ℝ(_value_)).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.instant.prototype.tostring step 6:
      6. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_987_650_000n);

let string = instant.toString({ fractionalSecondDigits: 2.5 });
assert.sameValue(string, "2001-09-09T01:46:40.98Z", "fractionalSecondDigits 2.5 floors to 2");

string = instant.toString({ fractionalSecondDigits: 9.7 });
assert.sameValue(string, "2001-09-09T01:46:40.987650000Z", "fractionalSecondDigits 9.7 floors to 9 and is not out of range");

assert.throws(
  RangeError,
  () => instant.toString({ fractionalSecondDigits: -0.6 }),
  "fractionalSecondDigits -0.6 floors to -1 and is out of range"
);
