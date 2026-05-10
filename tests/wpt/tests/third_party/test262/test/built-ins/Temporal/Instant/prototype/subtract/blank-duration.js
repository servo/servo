// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.subtract
description: Behaviour with blank duration
features: [Temporal]
---*/

const instant = new Temporal.Instant(1n);
const blank = new Temporal.Duration();
const result = instant.subtract(blank);
assert.sameValue(result.epochNanoseconds, 1n, "result is unchanged");
