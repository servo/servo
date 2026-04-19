// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Fallback value for roundingIncrement option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.duration.prototype.round step 18:
      18. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, _maximum_, *false*).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 4, 12, 34, 56, 987, 654, 321);

const explicit = duration.round({ smallestUnit: 'second', roundingIncrement: undefined });
TemporalHelpers.assertDuration(explicit, 0, 0, 0, 4, 12, 34, 57, 0, 0, 0, "default roundingIncrement is 1");

const implicit = duration.round({ smallestUnit: 'second' });
TemporalHelpers.assertDuration(implicit, 0, 0, 0, 4, 12, 34, 57, 0, 0, 0, "default roundingIncrement is 1");
