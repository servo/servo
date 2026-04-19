// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: RangeError thrown when roundingIncrement option is NaN
info: |
    sec-getoption step 8.b:
      b. If _value_ is *NaN*, throw a *RangeError* exception.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.instant.prototype.round step 13:
      13. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, _maximum_, *true*).
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_987_654_321n);
assert.throws(RangeError, () => instant.round({ smallestUnit: 'second', roundingIncrement: NaN }));
