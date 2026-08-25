// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Fallback value for roundingMode option
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_123_987_500n);

const explicit1 = instant.round({ smallestUnit: "microsecond", roundingMode: undefined });
assert.sameValue(explicit1.epochNanoseconds, 1_000_000_000_123_988_000n, "default roundingMode is halfExpand");
const implicit1 = instant.round({ smallestUnit: "microsecond" });
assert.sameValue(implicit1.epochNanoseconds, 1_000_000_000_123_988_000n, "default roundingMode is halfExpand");

const explicit2 = instant.round({ smallestUnit: "millisecond", roundingMode: undefined });
assert.sameValue(explicit2.epochNanoseconds, 1_000_000_000_124_000_000n, "default roundingMode is halfExpand");
const implicit2 = instant.round({ smallestUnit: "millisecond" });
assert.sameValue(implicit2.epochNanoseconds, 1_000_000_000_124_000_000n, "default roundingMode is halfExpand");

const explicit3 = instant.round({ smallestUnit: "second", roundingMode: undefined });
assert.sameValue(explicit3.epochNanoseconds, 1_000_000_000_000_000_000n, "default roundingMode is halfExpand");
const implicit3 = instant.round({ smallestUnit: "second" });
assert.sameValue(implicit3.epochNanoseconds, 1_000_000_000_000_000_000n, "default roundingMode is halfExpand");
