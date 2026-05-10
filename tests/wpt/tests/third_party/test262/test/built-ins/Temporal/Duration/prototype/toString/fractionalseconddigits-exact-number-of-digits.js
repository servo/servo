// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: |
  The fractionalSecondDigits and smallestUnit options determine the exact number
  of digits shown after the decimal separator, no more and no less
info: |
    sec-temporaldurationtostring step 21:
      21. If any of _seconds_, _milliseconds_, _microseconds_, and _nanoseconds_ are not 0; or _years_, _months_, _weeks_, _days_, _hours_, and _minutes_ are all 0; or _precision_ is not *"auto"*; then
features: [Temporal]
---*/

const threeYears = new Temporal.Duration(3);
assert.sameValue(threeYears.toString({ fractionalSecondDigits: 0 }), "P3YT0S");
assert.sameValue(threeYears.toString({ smallestUnit: 'seconds' }), "P3YT0S");
assert.sameValue(threeYears.toString({ smallestUnit: 'milliseconds' }), "P3YT0.000S");
assert.sameValue(threeYears.toString({ fractionalSecondDigits: 5 }), "P3YT0.00000S");

const halfHour = new Temporal.Duration(0, 0, 0, 0, 0, 30);
assert.sameValue(halfHour.toString({ fractionalSecondDigits: 0 }), "PT30M0S");
assert.sameValue(halfHour.toString({ smallestUnit: 'seconds' }), "PT30M0S");
assert.sameValue(halfHour.toString({ smallestUnit: 'milliseconds' }), "PT30M0.000S");
assert.sameValue(halfHour.toString({ fractionalSecondDigits: 5 }), "PT30M0.00000S");

const hundredMs = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 100);
assert.sameValue(hundredMs.toString({ fractionalSecondDigits: 0 }), "PT0S");
assert.sameValue(hundredMs.toString({ smallestUnit: 'seconds' }), "PT0S");
assert.sameValue(hundredMs.toString({ smallestUnit: 'milliseconds' }), "PT0.100S");
assert.sameValue(hundredMs.toString({ fractionalSecondDigits: 5 }), "PT0.10000S");

const zero = new Temporal.Duration();
assert.sameValue(zero.toString({ fractionalSecondDigits: 0 }), "PT0S");
assert.sameValue(zero.toString({ smallestUnit: 'seconds' }), "PT0S");
assert.sameValue(zero.toString({ smallestUnit: 'milliseconds' }), "PT0.000S");
assert.sameValue(zero.toString({ fractionalSecondDigits: 5 }), "PT0.00000S");
