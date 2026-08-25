// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: Fallback value for fractionalSecondDigits option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-getstringornumberoption step 2:
      2. Let _value_ be ? GetOption(_options_, _property_, « Number, String », *undefined*, _fallback_).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.plaintime.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainTime(5, 3, 1), "05:03:01"],
  [new Temporal.PlainTime(15, 23), "15:23:00"],
  [new Temporal.PlainTime(15, 23, 30), "15:23:30"],
  [new Temporal.PlainTime(15, 23, 30, 123, 400), "15:23:30.1234"],
];

for (const [time, expected] of tests) {
  const explicit = time.toString({ fractionalSecondDigits: undefined });
  assert.sameValue(explicit, expected, "default fractionalSecondDigits is auto (property present but undefined)");

  const implicit = time.toString({});
  assert.sameValue(implicit, expected, "default fractionalSecondDigits is auto (property not present)");

  const lambda = time.toString(() => {});
  assert.sameValue(lambda, expected, "default fractionalSecondDigits is auto (property not present, function object)");
}
