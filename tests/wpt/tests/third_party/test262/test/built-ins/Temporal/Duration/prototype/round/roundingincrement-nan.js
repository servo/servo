// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: RangeError thrown when roundingIncrement option is NaN
info: |
    sec-getoption step 8.b:
      b. If _value_ is *NaN*, throw a *RangeError* exception.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.duration.prototype.round step 18:
      18. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, _maximum_, *false*).
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 4, 12, 34, 56, 987, 654, 321);
assert.throws(RangeError, () => duration.round({ smallestUnit: 'second', roundingIncrement: NaN }));
