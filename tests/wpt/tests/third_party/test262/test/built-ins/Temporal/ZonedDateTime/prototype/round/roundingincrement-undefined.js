// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Fallback value for roundingIncrement option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal-totemporaldatetimeroundingincrement step 5:
      5. Return ? ToTemporalRoundingIncrement(_normalizedOptions_, _maximum_, *false*).
    sec-temporal.zoneddatetime.prototype.round step 8:
      8. Let _roundingIncrement_ be ? ToTemporalDateTimeRoundingIncrement(_options_, _smallestUnit_).
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");

const explicit = datetime.round({ smallestUnit: 'second', roundingIncrement: undefined });
assert.sameValue(explicit.epochNanoseconds, 1_000_000_001_000_000_000n, "default roundingIncrement is 1");

const implicit = datetime.round({ smallestUnit: 'second' });
assert.sameValue(implicit.epochNanoseconds, 1_000_000_001_000_000_000n, "default roundingIncrement is 1");
