// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Fallback value for disambiguation option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaldisambiguation step 1:
      1. Return ? GetOption(_normalizedOptions_, *"disambiguation"*, « String », « *"compatible"*, *"earlier"*, *"later"*, *"reject"* », *"compatible"*).
    sec-temporal.zoneddatetime.protoype.with step 14:
      14. Let _disambiguation_ be ? ToTemporalDisambiguation(_options_).
features: [Temporal]
---*/

const springForwardDatetime = new Temporal.ZonedDateTime(954702001_000_000_000n, "America/Vancouver");
const fallBackDatetime = new Temporal.ZonedDateTime(972849601_000_000_000n, "America/Vancouver");
const offset = "ignore";

[
  [springForwardDatetime, { hour: 2, minute: 30 }, 954671401_000_000_000n],
  [fallBackDatetime, { hour: 1, minute: 30 }, 972808201_000_000_000n],
].forEach(([datetime, fields, expected]) => {
  const explicit = datetime.with(fields, { offset, disambiguation: undefined });
  assert.sameValue(explicit.epochNanoseconds, expected, "default disambiguation is compatible");
  const implicit = datetime.with(fields, { offset });
  assert.sameValue(implicit.epochNanoseconds, expected, "default disambiguation is compatible");
});
