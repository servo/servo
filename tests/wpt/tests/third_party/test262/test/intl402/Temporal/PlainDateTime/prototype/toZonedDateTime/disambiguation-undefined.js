// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Fallback value for disambiguation option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaldisambiguation step 1:
      1. Return ? GetOption(_normalizedOptions_, *"disambiguation"*, « String », « *"compatible"*, *"earlier"*, *"later"*, *"reject"* », *"compatible"*).
    sec-temporal.plaindatetime.prototype.tozoneddatetime step 5:
      5. Let _disambiguation_ be ? ToTemporalDisambiguation(_options_).
features: [Temporal]
---*/

const timeZone = "America/Vancouver";
const springForwardDatetime = new Temporal.PlainDateTime(2000, 4, 2, 2, 30);
const fallBackDatetime = new Temporal.PlainDateTime(2000, 10, 29, 1, 30);

[
  [springForwardDatetime, 954671400_000_000_000n],
  [fallBackDatetime, 972808200_000_000_000n],
].forEach(([datetime, expected]) => {
  const explicit = datetime.toZonedDateTime(timeZone, { disambiguation: undefined });
  assert.sameValue(explicit.epochNanoseconds, expected, "default disambiguation is compatible");
  const implicit = datetime.toZonedDateTime(timeZone, {});
  assert.sameValue(implicit.epochNanoseconds, expected, "default disambiguation is compatible");
});
