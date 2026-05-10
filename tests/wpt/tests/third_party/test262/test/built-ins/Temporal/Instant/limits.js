// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant
description: Min/max range.
includes: [temporalHelpers.js]
features: [Temporal]
---*/


// constructing from ns
var limit = 8640000000000000000000n;
assert.throws(RangeError, () => new Temporal.Instant(-limit - 1n));
assert.throws(RangeError, () => new Temporal.Instant(limit + 1n));
TemporalHelpers.assertInstantsEqual(new Temporal.Instant(-limit),
                                    Temporal.Instant.from("-271821-04-20T00:00:00Z"));
TemporalHelpers.assertInstantsEqual(new Temporal.Instant(limit),
                                    Temporal.Instant.from("+275760-09-13T00:00:00Z"));
