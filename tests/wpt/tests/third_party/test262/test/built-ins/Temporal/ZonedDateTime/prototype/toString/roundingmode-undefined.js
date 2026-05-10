// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Fallback value for roundingMode option
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_123_987_500n, "UTC");

const explicit1 = datetime.toString({ smallestUnit: "microsecond", roundingMode: undefined });
assert.sameValue(explicit1, "2001-09-09T01:46:40.123987+00:00[UTC]", "default roundingMode is trunc");
const implicit1 = datetime.toString({ smallestUnit: "microsecond" });
assert.sameValue(implicit1, "2001-09-09T01:46:40.123987+00:00[UTC]", "default roundingMode is trunc");

const explicit2 = datetime.toString({ smallestUnit: "millisecond", roundingMode: undefined });
assert.sameValue(explicit2, "2001-09-09T01:46:40.123+00:00[UTC]", "default roundingMode is trunc");
const implicit2 = datetime.toString({ smallestUnit: "millisecond" });
assert.sameValue(implicit2, "2001-09-09T01:46:40.123+00:00[UTC]", "default roundingMode is trunc");

const explicit3 = datetime.toString({ smallestUnit: "second", roundingMode: undefined });
assert.sameValue(explicit3, "2001-09-09T01:46:40+00:00[UTC]", "default roundingMode is trunc");
const implicit3 = datetime.toString({ smallestUnit: "second" });
assert.sameValue(implicit3, "2001-09-09T01:46:40+00:00[UTC]", "default roundingMode is trunc");
