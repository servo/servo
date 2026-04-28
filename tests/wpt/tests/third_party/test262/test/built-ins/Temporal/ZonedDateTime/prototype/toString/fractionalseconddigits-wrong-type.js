// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Type conversions for fractionalSecondDigits option
info: |
    sec-getoption steps 8–9:
      8. Else if _type_ is Number, then
        a. Set _value_ to ? ToNumber(value).
        b. ...
      9. Else,
        a. Set _value_ to ? ToString(value).
    sec-getstringornumberoption step 2:
      2. Let _value_ be ? GetOption(_options_, _property_, « Number, String », *undefined*, _fallback_).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.zoneddatetime.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_650_000n, "UTC");

assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: null }),
  "null is not a number and converts to the string 'null' which is not valid for fractionalSecondDigits");
assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: true }),
  "true is not a number and converts to the string 'true' which is not valid for fractionalSecondDigits");
assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: false }),
  "false is not a number and converts to the string 'false' which is not valid for fractionalSecondDigits");
assert.throws(TypeError, () => datetime.toString({ fractionalSecondDigits: Symbol() }),
  "symbols are not numbers and cannot convert to strings");
assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: 2n }),
  "bigints are not numbers and convert to strings which are not valid for fractionalSecondDigits");
assert.throws(RangeError, () => datetime.toString({ fractionalSecondDigits: {} }),
  "plain objects are not numbers and convert to strings which are not valid for fractionalSecondDigits");

const expected = [
  "get fractionalSecondDigits.toString",
  "call fractionalSecondDigits.toString",
];
const actual = [];
const observer = TemporalHelpers.toPrimitiveObserver(actual, "auto", "fractionalSecondDigits");
const result = datetime.toString({ fractionalSecondDigits: observer });
assert.sameValue(result, "2001-09-09T01:46:40.98765+00:00[UTC]", "object with toString uses toString return value");
assert.compareArray(actual, expected, "object with toString calls toString and not valueOf");
