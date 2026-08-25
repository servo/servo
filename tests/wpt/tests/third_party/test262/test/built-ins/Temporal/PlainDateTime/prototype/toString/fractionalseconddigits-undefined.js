// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Fallback value for fractionalSecondDigits option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-getstringornumberoption step 2:
      2. Let _value_ be ? GetOption(_options_, _property_, « Number, String », *undefined*, _fallback_).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.plaindatetime.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainDateTime(1976, 2, 4, 5, 3, 1), "1976-02-04T05:03:01"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23), "1976-11-18T15:23:00"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30), "1976-11-18T15:23:30"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 400), "1976-11-18T15:23:30.1234"],
];

for (const [datetime, expected] of tests) {
  const explicit = datetime.toString({ fractionalSecondDigits: undefined });
  assert.sameValue(explicit, expected, "default fractionalSecondDigits is auto (property present but undefined)");

  const implicit = datetime.toString({});
  assert.sameValue(implicit, expected, "default fractionalSecondDigits is auto (property not present)");

  const lambda = datetime.toString(() => {});
  assert.sameValue(lambda, expected, "default fractionalSecondDigits is auto (property not present, function object)");
}
