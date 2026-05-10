// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.instant.prototype.epochmilliseconds
description: Basic tests for epochMilliseconds.
features: [BigInt, Temporal]
---*/

const afterEpoch = new Temporal.Instant(217175010_123_456_789n);
assert.sameValue(afterEpoch.epochMilliseconds, 217175010_123, "epochMilliseconds post epoch");
assert.sameValue(typeof afterEpoch.epochMilliseconds, "number", "epochMilliseconds value is a number");

const beforeEpoch = new Temporal.Instant(-217175010_876_543_211n);
assert.sameValue(beforeEpoch.epochMilliseconds, -217175010_877, "epochMilliseconds pre epoch");
assert.sameValue(typeof beforeEpoch.epochMilliseconds, "number", "epochMilliseconds value is a number");
