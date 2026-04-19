// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Fallback value for largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 0, 0, 0);
const later = new Temporal.PlainDateTime(2001, 6, 3, 13, 35, 57, 987, 654, 321);

const explicit = later.since(earlier, { largestUnit: undefined });
TemporalHelpers.assertDuration(explicit, 0, 0, 0, 397, 1, 1, 1, 987, 654, 321, "default largestUnit is day");
const implicit = later.since(earlier, {});
TemporalHelpers.assertDuration(implicit, 0, 0, 0, 397, 1, 1, 1, 987, 654, 321, "default largestUnit is day");
const lambda = later.since(earlier, () => {});
TemporalHelpers.assertDuration(lambda, 0, 0, 0, 397, 1, 1, 1, 987, 654, 321, "default largestUnit is day");
