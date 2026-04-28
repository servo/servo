// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Fallback value for smallestUnit option
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_123_987_500n, "UTC");

const explicit1 = datetime.toString({ smallestUnit: undefined, fractionalSecondDigits: 6 });
assert.sameValue(explicit1, "2001-09-09T01:46:40.123987+00:00[UTC]", "default smallestUnit defers to fractionalSecondDigits");
const implicit1 = datetime.toString({ fractionalSecondDigits: 6 });
assert.sameValue(implicit1, "2001-09-09T01:46:40.123987+00:00[UTC]", "default smallestUnit defers to fractionalSecondDigits");

const explicit2 = datetime.toString({ smallestUnit: undefined, fractionalSecondDigits: 3 });
assert.sameValue(explicit2, "2001-09-09T01:46:40.123+00:00[UTC]", "default smallestUnit defers to fractionalSecondDigits");
const implicit2 = datetime.toString({ fractionalSecondDigits: 3 });
assert.sameValue(implicit2, "2001-09-09T01:46:40.123+00:00[UTC]", "default smallestUnit defers to fractionalSecondDigits");
