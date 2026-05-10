// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Fallback value for fractionalSecondDigits option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-getstringornumberoption step 2:
      2. Let _value_ be ? GetOption(_options_, _property_, « Number, String », *undefined*, _fallback_).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.duration.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const wholeSeconds = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7);
const subSeconds = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 650);

const tests = [
  [wholeSeconds, "P1Y2M3W4DT5H6M7S"],
  [subSeconds, "P1Y2M3W4DT5H6M7.98765S"],
];

for (const [duration, expected] of tests) {
  const explicit = duration.toString({ fractionalSecondDigits: undefined });
  assert.sameValue(explicit, expected, "default fractionalSecondDigits is auto (property present but undefined)");

  const implicit = duration.toString({});
  assert.sameValue(implicit, expected, "default fractionalSecondDigits is auto (property not present)");

  const lambda = duration.toString(() => {});
  assert.sameValue(lambda, expected, "default fractionalSecondDigits is auto (property not present, function object)");
}
