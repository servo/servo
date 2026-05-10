// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.add
description: >
  Instant is minimum/maximum instant.
features: [Temporal]
---*/

let min = new Temporal.Instant(-86_40000_00000_00000_00000n);
let max = new Temporal.Instant(86_40000_00000_00000_00000n);

let zero = Temporal.Duration.from({nanoseconds: 0});
let one = Temporal.Duration.from({nanoseconds: 1});
let minusOne = Temporal.Duration.from({nanoseconds: -1});

assert.sameValue(min.add(zero).epochNanoseconds, min.epochNanoseconds,
                 "Adding zero to the minimum instant");

assert.sameValue(max.add(zero).epochNanoseconds, max.epochNanoseconds,
                 "Adding zero to the maximum instant");

assert.throws(RangeError, () => min.add(minusOne),
              "Subtracting one from the minimum instant");

assert.throws(RangeError, () => max.add(one),
              "Adding one to the maximum instant");

assert.sameValue(min.add(one).epochNanoseconds, min.epochNanoseconds + 1n,
                 "Adding one to the minimum instant");

assert.sameValue(max.add(minusOne).epochNanoseconds, max.epochNanoseconds - 1n,
                 "Subtracting one from the maximum instant");

// From minimum to maximum instant.
assert.sameValue(min.add({nanoseconds: 86_40000_00000_00000_00000 * 2}).epochNanoseconds, max.epochNanoseconds,
                 "Minimum to maximum instant by adding nanoseconds");

assert.sameValue(min.add({microseconds: 8640_00000_00000_00000 * 2}).epochNanoseconds, max.epochNanoseconds,
                 "Minimum to maximum instant by adding microseconds");

assert.sameValue(min.add({milliseconds: 8_64000_00000_00000 * 2}).epochNanoseconds, max.epochNanoseconds,
                 "Minimum to maximum instant by adding milliseconds");

assert.sameValue(min.add({seconds: 864_00000_00000 * 2}).epochNanoseconds, max.epochNanoseconds,
                 "Minimum to maximum instant by adding seconds");

// From maximum to minimum instant.
assert.sameValue(max.add({nanoseconds: -86_40000_00000_00000_00000 * 2}).epochNanoseconds, min.epochNanoseconds,
                 "Maximum to minimum instant by adding nanoseconds");

assert.sameValue(max.add({microseconds: -8640_00000_00000_00000 * 2}).epochNanoseconds, min.epochNanoseconds,
                 "Maximum to minimum instant by adding microseconds");

assert.sameValue(max.add({milliseconds: -8_64000_00000_00000 * 2}).epochNanoseconds, min.epochNanoseconds,
                 "Maximum to minimum instant by adding milliseconds");

assert.sameValue(max.add({seconds: -864_00000_00000 * 2}).epochNanoseconds, min.epochNanoseconds,
                 "Maximum to minimum instant by adding seconds");
