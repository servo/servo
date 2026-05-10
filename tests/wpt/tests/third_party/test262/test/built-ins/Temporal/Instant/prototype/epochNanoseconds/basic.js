// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.instant.prototype.epochnanoseconds
description: Basic tests for epochNanoseconds.
features: [BigInt, Temporal]
---*/

const afterEpoch = new Temporal.Instant(217175010_123_456_789n);
assert.sameValue(afterEpoch.epochNanoseconds, 217175010_123_456_789n, "epochNanoseconds post epoch");
assert.sameValue(typeof afterEpoch.epochNanoseconds, "bigint", "epochNanoseconds value is a bigint");

const beforeEpoch = new Temporal.Instant(-217175010_876_543_211n);
assert.sameValue(beforeEpoch.epochNanoseconds, -217175010_876_543_211n, "epochNanoseconds pre epoch");
assert.sameValue(typeof beforeEpoch.epochNanoseconds, "bigint", "epochNanoseconds value is a bigint");
