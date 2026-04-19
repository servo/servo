// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Rounding can cross unit boundaries up to largestUnit
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.Instant(0n);
const later = new Temporal.Instant(7199_000_000_000n);
const duration = earlier.since(later, { largestUnit: "hours", smallestUnit: "minutes", roundingMode: "expand" });
TemporalHelpers.assertDuration(duration, 0, 0, 0, 0, -2, 0, 0, 0, 0, 0, "-1:59 balances to -2 hours");
