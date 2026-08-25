// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: RangeError thrown when roundingIncrement option out of range
info: |
    ToTemporalRoundingIncrement ( _normalizedOptions_ )

    1. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, *"number"*, *undefined*, *1*<sub>𝔽</sub>).
    2. If _increment_ is not finite, throw a *RangeError* exception.
    3. Let _integerIncrement_ be truncate(ℝ(_increment_)).
    4. If _integerIncrement_ < 1 or _integerIncrement_ > 10<sup>9</sup>, throw a *RangeError* exception.
    5. Return _integerIncrement_.
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2000, 10);
assert.throws(RangeError, () => earlier.until(later, { roundingIncrement: -Infinity }));
assert.throws(RangeError, () => earlier.until(later, { roundingIncrement: -1 }));
assert.throws(RangeError, () => earlier.until(later, { roundingIncrement: 0 }));
assert.throws(RangeError, () => earlier.until(later, { roundingIncrement: 0.9 }));
assert.throws(RangeError, () => earlier.until(later, { roundingIncrement: 1e9 + 1 }));
assert.throws(RangeError, () => earlier.until(later, { roundingIncrement: Infinity }));
