// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.fromepochnanoseconds
description: Basic tests for Instant.fromEpochNanoseconds().
features: [BigInt, Temporal]
---*/

const afterEpoch = Temporal.Instant.fromEpochNanoseconds(217175010_123_456_789n);
assert.sameValue(afterEpoch.epochNanoseconds, 217175010_123_456_789n, "fromEpochNanoseconds post epoch");

const beforeEpoch = Temporal.Instant.fromEpochNanoseconds(-217175010_876_543_211n);
assert.sameValue(beforeEpoch.epochNanoseconds, -217175010_876_543_211n, "fromEpochNanoseconds pre epoch");
