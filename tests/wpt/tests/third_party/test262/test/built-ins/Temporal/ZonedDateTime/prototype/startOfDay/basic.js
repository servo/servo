// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.startofday
description: Basic functionality
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(10000n * 86400_000_000_000n + 7272_123_456_789n, "UTC");
const result = instance.startOfDay();
assert.sameValue(result.epochNanoseconds, 10000n * 86400_000_000_000n);
