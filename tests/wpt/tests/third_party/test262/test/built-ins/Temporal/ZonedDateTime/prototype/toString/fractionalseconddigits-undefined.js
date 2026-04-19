// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Fallback value for fractionalSecondDigits option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-getstringornumberoption step 2:
      2. Let _value_ be ? GetOption(_options_, _property_, « Number, String », *undefined*, _fallback_).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.zoneddatetime.prototype.tostring step 4:
      4. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const tests = [
  [new Temporal.ZonedDateTime(192_258_181_000_000_000n, "UTC"), "1976-02-04T05:03:01+00:00[UTC]"],
  [new Temporal.ZonedDateTime(0n, "UTC"), "1970-01-01T00:00:00+00:00[UTC]"],
  [new Temporal.ZonedDateTime(30_000_000_000n, "UTC"), "1970-01-01T00:00:30+00:00[UTC]"],
  [new Temporal.ZonedDateTime(30_123_400_000n, "UTC"), "1970-01-01T00:00:30.1234+00:00[UTC]"],
];

for (const [datetime, expected] of tests) {
  const explicit = datetime.toString({ fractionalSecondDigits: undefined });
  assert.sameValue(explicit, expected, "default fractionalSecondDigits is auto (property present but undefined)");

  const implicit = datetime.toString({});
  assert.sameValue(implicit, expected, "default fractionalSecondDigits is auto (property not present)");

  const lambda = datetime.toString(() => {});
  assert.sameValue(lambda, expected, "default fractionalSecondDigits is auto (property not present, function object)");
}
