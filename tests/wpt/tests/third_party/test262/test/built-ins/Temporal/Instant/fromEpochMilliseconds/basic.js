// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.fromepochmilliseconds
description: Basic tests for Instant.fromEpochMilliseconds().
features: [BigInt, Temporal]
---*/

const afterEpoch = Temporal.Instant.fromEpochMilliseconds(217175010_123);
assert.sameValue(afterEpoch.epochNanoseconds, 217175010_123_000_000n, "fromEpochMilliseconds post epoch");

const beforeEpoch = Temporal.Instant.fromEpochMilliseconds(-217175010_876);
assert.sameValue(beforeEpoch.epochNanoseconds, -217175010_876_000_000n, "fromEpochMilliseconds pre epoch");
