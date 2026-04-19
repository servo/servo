// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Fallback value for roundingIncrement option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.instant.prototype.round step 13:
      13. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, _maximum_, *true*).
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_987_654_321n);

const explicit = instant.round({ smallestUnit: 'second', roundingIncrement: undefined });
assert.sameValue(explicit.epochNanoseconds, 1_000_000_001_000_000_000n, "default roundingIncrement is 1");

const implicit = instant.round({ smallestUnit: 'second' });
assert.sameValue(implicit.epochNanoseconds, 1_000_000_001_000_000_000n, "default roundingIncrement is 1");
