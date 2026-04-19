// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: RangeError thrown when roundingIncrement option out of range
info: |
    sec-temporal-totemporalroundingincrement:
      4. If _integerIncrement_ < 1 or _integerIncrement_ > 10<sup>9</sup>, throw a *RangeError* exception.
features: [Temporal]
---*/

const instance = new Temporal.Duration(1);
const options = { smallestUnit: "years", relativeTo: new Temporal.PlainDate(2000, 1, 1) };
assert.throws(RangeError, () => instance.round({ ...options, roundingIncrement: -Infinity }));
assert.throws(RangeError, () => instance.round({ ...options, roundingIncrement: -1 }));
assert.throws(RangeError, () => instance.round({ ...options, roundingIncrement: 0 }));
assert.throws(RangeError, () => instance.round({ ...options, roundingIncrement: 1e9 + 1 }));
assert.throws(RangeError, () => instance.round({ ...options, roundingIncrement: Infinity }));
