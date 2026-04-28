// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Number for fractionalSecondDigits option
features: [Temporal]
---*/

const wholeSeconds = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7);
const subSeconds = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 650);

assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 0 }), "P1Y2M3W4DT5H6M7S",
  "truncates 4 decimal places to 0");
assert.sameValue(wholeSeconds.toString({ fractionalSecondDigits: 2 }), "P1Y2M3W4DT5H6M7.00S",
  "pads whole seconds to 2 decimal places");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 2 }), "P1Y2M3W4DT5H6M7.98S",
  "truncates 4 decimal places to 2");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 3 }), "P1Y2M3W4DT5H6M7.987S",
  "truncates 4 decimal places to 3");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 6 }), "P1Y2M3W4DT5H6M7.987650S",
  "pads 4 decimal places to 6");
assert.sameValue(wholeSeconds.toString({ fractionalSecondDigits: 7 }), "P1Y2M3W4DT5H6M7.0000000S",
  "pads whole seconds to 7 decimal places");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 7 }), "P1Y2M3W4DT5H6M7.9876500S",
  "pads 4 decimal places to 7");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 9 }), "P1Y2M3W4DT5H6M7.987650000S",
  "pads 4 decimal places to 9");
