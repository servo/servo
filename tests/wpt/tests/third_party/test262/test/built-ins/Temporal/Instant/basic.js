// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant
description: Basic functionality of the Temporal.Instant constructor
features: [Temporal]
---*/

const bigIntInstant = new Temporal.Instant(217175010123456789n);
assert(bigIntInstant instanceof Temporal.Instant, "BigInt instanceof");
assert.sameValue(bigIntInstant.epochMilliseconds, 217175010123, "BigInt epochMilliseconds");
assert.sameValue(bigIntInstant.epochNanoseconds, 217175010123456789n, "BigInt epochNanoseconds");

const stringInstant = new Temporal.Instant("217175010123456789");
assert(stringInstant instanceof Temporal.Instant, "String instanceof");
assert.sameValue(stringInstant.epochMilliseconds, 217175010123, "String epochMilliseconds");
assert.sameValue(stringInstant.epochNanoseconds, 217175010123456789n, "String epochNanoseconds");

const negativeBigIntInstant = new Temporal.Instant(-217175010123456789n);
assert(negativeBigIntInstant instanceof Temporal.Instant, "negative BigInt instanceof");
assert.sameValue(negativeBigIntInstant.epochMilliseconds, -217175010124, "negagive BigInt epochMilliseconds");
assert.sameValue(negativeBigIntInstant.epochNanoseconds, -217175010123456789n, "negative BigInt epochNanoseconds");

const negativeStringInstant = new Temporal.Instant("-217175010123456789");
assert(negativeStringInstant instanceof Temporal.Instant, "negative String instanceof");
assert.sameValue(negativeStringInstant.epochMilliseconds, -217175010124, "negative string epochMilliseconds");
assert.sameValue(negativeStringInstant.epochNanoseconds, -217175010123456789n, "negative string epochNanoseconds");

assert.throws(SyntaxError, () => new Temporal.Instant("abc123"), "invalid BigInt syntax");

assert.sameValue(new Temporal.Instant(true).epochNanoseconds, 1n, "true as argument is 1n");
assert.sameValue(new Temporal.Instant(false).epochNanoseconds, 0n, "false as argument is 0n");
