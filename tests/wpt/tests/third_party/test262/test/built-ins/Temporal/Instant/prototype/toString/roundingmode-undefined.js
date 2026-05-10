// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Fallback value for roundingMode option
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_123_987_500n);

const explicit1 = instant.toString({ smallestUnit: "microsecond", roundingMode: undefined });
assert.sameValue(explicit1, "2001-09-09T01:46:40.123987Z", "default roundingMode is trunc");
const implicit1 = instant.toString({ smallestUnit: "microsecond" });
assert.sameValue(implicit1, "2001-09-09T01:46:40.123987Z", "default roundingMode is trunc");

const explicit2 = instant.toString({ smallestUnit: "millisecond", roundingMode: undefined });
assert.sameValue(explicit2, "2001-09-09T01:46:40.123Z", "default roundingMode is trunc");
const implicit2 = instant.toString({ smallestUnit: "millisecond" });
assert.sameValue(implicit2, "2001-09-09T01:46:40.123Z", "default roundingMode is trunc");

const explicit3 = instant.toString({ smallestUnit: "second", roundingMode: undefined });
assert.sameValue(explicit3, "2001-09-09T01:46:40Z", "default roundingMode is trunc");
const implicit3 = instant.toString({ smallestUnit: "second" });
assert.sameValue(implicit3, "2001-09-09T01:46:40Z", "default roundingMode is trunc");
