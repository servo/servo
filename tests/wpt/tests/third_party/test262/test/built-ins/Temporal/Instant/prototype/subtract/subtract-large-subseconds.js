// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.subtract
description: Subtracting unbalanced durations with large subsecond values from a date
features: [Temporal]
---*/

const i1 = new Temporal.Instant(1582966647747612578n);

assert.sameValue(i1.subtract(Temporal.Duration.from({nanoseconds: Number.MAX_SAFE_INTEGER})).epochNanoseconds,
                 1573959448492871587n);
assert.sameValue(i1.subtract(Temporal.Duration.from({nanoseconds: Number.MIN_SAFE_INTEGER})).epochNanoseconds,
                 1591973847002353569n);

assert.sameValue(i1.subtract(Temporal.Duration.from({microseconds: Number.MAX_SAFE_INTEGER})).epochNanoseconds,
                 -7424232606993378422n);
assert.sameValue(i1.subtract(Temporal.Duration.from({microseconds: Number.MIN_SAFE_INTEGER})).epochNanoseconds,
                 10590165902488603578n);

assert.throws(RangeError, () => i1.subtract(Temporal.Duration.from({milliseconds: Number.MAX_SAFE_INTEGER})));
assert.throws(RangeError, () => i1.subtract(Temporal.Duration.from({milliseconds: Number.MIN_SAFE_INTEGER})));

assert.throws(RangeError, () => i1.subtract(Temporal.Duration.from({seconds: Number.MAX_SAFE_INTEGER})));
assert.throws(RangeError, () => i1.subtract(Temporal.Duration.from({seconds: Number.MIN_SAFE_INTEGER})));

const bigNumber = 9007199254740990976;

assert.sameValue(i1.subtract(Temporal.Duration.from({nanoseconds: bigNumber})).epochNanoseconds,
                 -7424232606993378398n);
assert.sameValue(i1.subtract(Temporal.Duration.from({nanoseconds: -bigNumber})).epochNanoseconds,
                 10590165902488603554n);

assert.throws(RangeError, () => i1.subtract(Temporal.Duration.from({microseconds: bigNumber})));
assert.throws(RangeError, () => i1.subtract(Temporal.Duration.from({microseconds: -bigNumber})));

assert.throws(RangeError, () => i1.subtract(Temporal.Duration.from({milliseconds: bigNumber})));
assert.throws(RangeError, () => i1.subtract(Temporal.Duration.from({milliseconds: -bigNumber})));

const i2 = new Temporal.Instant(0n);

assert.sameValue(i2.subtract(Temporal.Duration.from({nanoseconds: bigNumber})).epochNanoseconds,
                 -9007199254740990976n);
assert.sameValue(i2.subtract(Temporal.Duration.from({nanoseconds: -bigNumber})).epochNanoseconds,
                 9007199254740990976n);

assert.throws(RangeError, () => i2.subtract(Temporal.Duration.from({microseconds: bigNumber})));
assert.throws(RangeError, () => i2.subtract(Temporal.Duration.from({microseconds: -bigNumber})));

assert.throws(RangeError, () => i2.subtract(Temporal.Duration.from({milliseconds: bigNumber})));
assert.throws(RangeError, () => i2.subtract(Temporal.Duration.from({milliseconds: -bigNumber})));
