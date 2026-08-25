// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Fallback value for disambiguation option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaldisambiguation step 1:
      1. Return ? GetOption(_normalizedOptions_, *"disambiguation"*, « String », « *"compatible"*, *"earlier"*, *"later"*, *"reject"* », *"compatible"*).
    sec-temporal-totemporalzoneddatetime step 5:
      5. Let _disambiguation_ be ? ToTemporalDisambiguation(_options_).
    sec-temporal.zoneddatetime.from step 2:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalZonedDateTime]] internal slot, then
        ...
        d. Return ...
      3. Return ? ToTemporalZonedDateTime(_item_, _options_).
features: [Temporal]
---*/

const springForwardFields = { timeZone: "America/Vancouver", year: 2000, month: 4, day: 2, hour: 2, minute: 30 };
const fallBackFields = { timeZone: "America/Vancouver", year: 2000, month: 10, day: 29, hour: 1, minute: 30 };

[
  [springForwardFields, 954671400_000_000_000n],
  [fallBackFields, 972808200_000_000_000n],
].forEach(([fields, expected]) => {
  const explicit = Temporal.ZonedDateTime.from(fields, { disambiguation: undefined });
  assert.sameValue(explicit.epochNanoseconds, expected, "default disambiguation is compatible (later)");

  // See options-undefined.js for {}
});
