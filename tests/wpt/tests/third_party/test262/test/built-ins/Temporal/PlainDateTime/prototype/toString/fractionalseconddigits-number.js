// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Number for fractionalSecondDigits option
features: [Temporal]
---*/

const fewSeconds = new Temporal.PlainDateTime(1976, 2, 4, 5, 3, 1);
const zeroSeconds = new Temporal.PlainDateTime(1976, 11, 18, 15, 23);
const wholeSeconds = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30);
const subSeconds = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 400);

assert.sameValue(fewSeconds.toString({ fractionalSecondDigits: 0 }), "1976-02-04T05:03:01",
  "pads parts with 0");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 0 }), "1976-11-18T15:23:30",
  "truncates 4 decimal places to 0");
assert.sameValue(zeroSeconds.toString({ fractionalSecondDigits: 1 }), "1976-11-18T15:23:00.0",
  "pads zero seconds to 1 decimal place");
assert.sameValue(zeroSeconds.toString({ fractionalSecondDigits: 2 }), "1976-11-18T15:23:00.00",
  "pads zero seconds to 2 decimal places");
assert.sameValue(wholeSeconds.toString({ fractionalSecondDigits: 2 }), "1976-11-18T15:23:30.00",
  "pads whole seconds to 2 decimal places");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 2 }), "1976-11-18T15:23:30.12",
  "truncates 4 decimal places to 2");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 3 }), "1976-11-18T15:23:30.123",
  "truncates 4 decimal places to 3");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 4 }), "1976-11-18T15:23:30.1234",
  "does not pad");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 5 }), "1976-11-18T15:23:30.12340",
  "pads 4 decimal places to 5");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 6 }), "1976-11-18T15:23:30.123400",
  "pads 4 decimal places to 6");
assert.sameValue(zeroSeconds.toString({ fractionalSecondDigits: 7 }), "1976-11-18T15:23:00.0000000",
  "pads zero seconds to 7 decimal places");
assert.sameValue(wholeSeconds.toString({ fractionalSecondDigits: 7 }), "1976-11-18T15:23:30.0000000",
  "pads whole seconds to 7 decimal places");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 7 }), "1976-11-18T15:23:30.1234000",
  "pads 4 decimal places to 7");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 8 }), "1976-11-18T15:23:30.12340000",
  "pads 4 decimal places to 8");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 9 }), "1976-11-18T15:23:30.123400000",
  "pads 4 decimal places to 9");
