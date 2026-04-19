// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Behaviour with blank duration
features: [Temporal]
---*/

const dt = new Temporal.ZonedDateTime(1n, "UTC");
const blank = new Temporal.Duration();
const result = dt.subtract(blank);
assert.sameValue(result.epochNanoseconds, 1n, "result is unchanged");
