// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - Temporal
description: |
  pending
esid: pending
---*/

const min = new Date(-8640000000000000).toTemporalInstant();
const max = new Date(8640000000000000).toTemporalInstant();
const epoch = new Date(0).toTemporalInstant();

const minTemporalInstant = new Temporal.Instant(-8640000000000000000000n)
const maxTemporalInstant = new Temporal.Instant(8640000000000000000000n)
const zeroInstant = new Temporal.Instant(0n)

let zero = Temporal.Duration.from({nanoseconds: 0});
let one = Temporal.Duration.from({nanoseconds: 1});
let minusOne = Temporal.Duration.from({nanoseconds: -1});

//Test invalid date
{
    const invalidDate = new Date(NaN);
    assert.throws(RangeError, () => invalidDate.toTemporalInstant());
}

//Test Temporal.Instant properties
{
    // epochNanoseconds
    assert.sameValue(min.epochNanoseconds, minTemporalInstant.epochNanoseconds);
    assert.sameValue(max.epochNanoseconds, maxTemporalInstant.epochNanoseconds);
    assert.sameValue(epoch.epochNanoseconds, zeroInstant.epochNanoseconds);

    // toZonedDateTime
    assert.sameValue(min.toZonedDateTimeISO('UTC').toString(), minTemporalInstant.toZonedDateTimeISO('UTC').toString());
    assert.sameValue(max.toZonedDateTimeISO('UTC').toString(), maxTemporalInstant.toZonedDateTimeISO('UTC').toString());
    assert.sameValue(epoch.toZonedDateTimeISO('UTC').toString(), zeroInstant.toZonedDateTimeISO('UTC').toString());
}

// Test values around the minimum/maximum instant.
{
    // Adding zero to the minimum instant.
    assert.sameValue(min.add(zero).epochNanoseconds, min.epochNanoseconds);
    assert.sameValue(min.subtract(zero).epochNanoseconds, min.epochNanoseconds);

    // Adding zero to the maximum instant.
    assert.sameValue(max.add(zero).epochNanoseconds, max.epochNanoseconds);
    assert.sameValue(max.subtract(zero).epochNanoseconds, max.epochNanoseconds);

    // Adding one to the minimum instant.
    assert.sameValue(min.add(one).epochNanoseconds, min.epochNanoseconds + 1n);
    assert.sameValue(min.subtract(minusOne).epochNanoseconds, min.epochNanoseconds + 1n);

    // Subtracting one from the maximum instant.
    assert.sameValue(max.add(minusOne).epochNanoseconds, max.epochNanoseconds - 1n);
    assert.sameValue(max.subtract(one).epochNanoseconds, max.epochNanoseconds - 1n);

    // Subtracting one from the minimum instant.
    assert.throws(RangeError, () => min.add(minusOne));
    assert.throws(RangeError, () => min.subtract(one));

    // Adding one to the maximum instant.
    assert.throws(RangeError, () => max.add(one));
    assert.throws(RangeError, () => max.subtract(minusOne));
}

