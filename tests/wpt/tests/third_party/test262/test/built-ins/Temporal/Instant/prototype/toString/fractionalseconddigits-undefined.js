// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Fallback value for fractionalSecondDigits option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-getstringornumberoption step 2:
      2. Let _value_ be ? GetOption(_options_, _property_, « Number, String », *undefined*, _fallback_).
    sec-temporal-tosecondsstringprecision step 9:
      9. Let _digits_ be ? GetStringOrNumberOption(_normalizedOptions_, *"fractionalSecondDigits"*, « *"auto"* », 0, 9, *"auto"*).
    sec-temporal.instant.prototype.tostring step 6:
      6. Let _precision_ be ? ToSecondsStringPrecision(_options_).
features: [Temporal]
---*/

const tests = [
  [new Temporal.Instant(192_258_181_000_000_000n), "1976-02-04T05:03:01Z"],
  [new Temporal.Instant(0n), "1970-01-01T00:00:00Z"],
  [new Temporal.Instant(30_000_000_000n), "1970-01-01T00:00:30Z"],
  [new Temporal.Instant(30_123_400_000n), "1970-01-01T00:00:30.1234Z"],
];

for (const [instant, expected] of tests) {
  const explicit = instant.toString({ fractionalSecondDigits: undefined });
  assert.sameValue(explicit, expected, "default fractionalSecondDigits is auto (property present but undefined)");

  const implicit = instant.toString({});
  assert.sameValue(implicit, expected, "default fractionalSecondDigits is auto (property not present)");

  const lambda = instant.toString(() => {});
  assert.sameValue(lambda, expected, "default fractionalSecondDigits is auto (property not present, function object)");
}
