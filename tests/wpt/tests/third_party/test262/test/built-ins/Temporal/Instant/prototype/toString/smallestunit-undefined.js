// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Fallback value for smallestUnit option
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_123_987_500n);

const explicit1 = instant.toString({ smallestUnit: undefined, fractionalSecondDigits: 6 });
assert.sameValue(explicit1, "2001-09-09T01:46:40.123987Z", "default smallestUnit defers to fractionalSecondDigits");
const implicit1 = instant.toString({ fractionalSecondDigits: 6 });
assert.sameValue(implicit1, "2001-09-09T01:46:40.123987Z", "default smallestUnit defers to fractionalSecondDigits");

const explicit2 = instant.toString({ smallestUnit: undefined, fractionalSecondDigits: 3 });
assert.sameValue(explicit2, "2001-09-09T01:46:40.123Z", "default smallestUnit defers to fractionalSecondDigits");
const implicit2 = instant.toString({ fractionalSecondDigits: 3 });
assert.sameValue(implicit2, "2001-09-09T01:46:40.123Z", "default smallestUnit defers to fractionalSecondDigits");
