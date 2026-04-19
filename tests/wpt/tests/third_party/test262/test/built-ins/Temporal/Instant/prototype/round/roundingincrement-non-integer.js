// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Rounding for roundingIncrement option
info: |
    ToTemporalRoundingIncrement ( _normalizedOptions_ )

    1. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, *"number"*, *undefined*, *1*<sub>𝔽</sub>).
    2. If _increment_ is not finite, throw a *RangeError* exception.
    3. Let _integerIncrement_ be truncate(ℝ(_increment_)).
    4. If _integerIncrement_ < 1 or _integerIncrement_ > 10<sup>9</sup>, throw a *RangeError* exception.
    5. Return _integerIncrement_.
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_000_000_005n);
const result = instant.round({ smallestUnit: "nanosecond", roundingIncrement: 2.5, roundingMode: "expand" });
assert.sameValue(result.epochNanoseconds, 1_000_000_000_000_000_006n, "roundingIncrement 2.5 truncates to 2");
