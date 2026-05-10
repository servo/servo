// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Fallback value for roundingMode option
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_123_987_500n, "UTC");

const explicit1 = datetime.round({ smallestUnit: "microsecond", roundingMode: undefined });
assert.sameValue(explicit1.epochNanoseconds, 1_000_000_000_123_988_000n, "default roundingMode is halfExpand");
const implicit1 = datetime.round({ smallestUnit: "microsecond" });
assert.sameValue(implicit1.epochNanoseconds, 1_000_000_000_123_988_000n, "default roundingMode is halfExpand");

const explicit2 = datetime.round({ smallestUnit: "millisecond", roundingMode: undefined });
assert.sameValue(explicit2.epochNanoseconds, 1_000_000_000_124_000_000n, "default roundingMode is halfExpand");
const implicit2 = datetime.round({ smallestUnit: "millisecond" });
assert.sameValue(implicit2.epochNanoseconds, 1_000_000_000_124_000_000n, "default roundingMode is halfExpand");

const explicit3 = datetime.round({ smallestUnit: "second", roundingMode: undefined });
assert.sameValue(explicit3.epochNanoseconds, 1_000_000_000_000_000_000n, "default roundingMode is halfExpand");
const implicit3 = datetime.round({ smallestUnit: "second" });
assert.sameValue(implicit3.epochNanoseconds, 1_000_000_000_000_000_000n, "default roundingMode is halfExpand");
