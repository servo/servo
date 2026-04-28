// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.subtract
description: >
  Instant is minimum/maximum instant.
features: [Temporal]
---*/

let min = new Temporal.Instant(-86_40000_00000_00000_00000n);
let max = new Temporal.Instant(86_40000_00000_00000_00000n);

let zero = Temporal.Duration.from({nanoseconds: 0});
let one = Temporal.Duration.from({nanoseconds: 1});
let minusOne = Temporal.Duration.from({nanoseconds: -1});

// Adding zero to the minimum instant.
assert.sameValue(min.subtract(zero).epochNanoseconds, min.epochNanoseconds);

// Adding zero to the maximum instant.
assert.sameValue(max.subtract(zero).epochNanoseconds, max.epochNanoseconds);

// Subtracting one from the minimum instant.
assert.throws(RangeError, () => min.subtract(one));

// Adding one to the maximum instant.
assert.throws(RangeError, () => max.subtract(minusOne));

// Adding one to the minimum instant.
assert.sameValue(min.subtract(minusOne).epochNanoseconds, min.epochNanoseconds + 1n);

// Subtracting one from the maximum instant.
assert.sameValue(max.subtract(one).epochNanoseconds, max.epochNanoseconds - 1n);

// From minimum to maximum instant.
assert.sameValue(min.subtract({nanoseconds: -86_40000_00000_00000_00000 * 2}).epochNanoseconds, max.epochNanoseconds);

assert.sameValue(min.subtract({microseconds: -8640_00000_00000_00000 * 2}).epochNanoseconds, max.epochNanoseconds);

assert.sameValue(min.subtract({milliseconds: -8_64000_00000_00000 * 2}).epochNanoseconds, max.epochNanoseconds);

assert.sameValue(min.subtract({seconds: -864_00000_00000 * 2}).epochNanoseconds, max.epochNanoseconds);

// From maximum to minimum instant.
assert.sameValue(max.subtract({nanoseconds: 86_40000_00000_00000_00000 * 2}).epochNanoseconds, min.epochNanoseconds);

assert.sameValue(max.subtract({microseconds: 8640_00000_00000_00000 * 2}).epochNanoseconds, min.epochNanoseconds);

assert.sameValue(max.subtract({milliseconds: 8_64000_00000_00000 * 2}).epochNanoseconds, min.epochNanoseconds);

assert.sameValue(max.subtract({seconds: 864_00000_00000 * 2}).epochNanoseconds, min.epochNanoseconds);
