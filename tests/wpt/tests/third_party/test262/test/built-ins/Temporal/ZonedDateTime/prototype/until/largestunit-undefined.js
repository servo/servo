// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Fallback value for largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");
const later = new Temporal.ZonedDateTime(1_000_090_061_987_654_321n, "UTC");

const explicit = earlier.until(later, { largestUnit: undefined });
TemporalHelpers.assertDuration(explicit, 0, 0, 0, 0, 25, 1, 1, 987, 654, 321, "default largestUnit is hour");
const implicit = earlier.until(later, {});
TemporalHelpers.assertDuration(implicit, 0, 0, 0, 0, 25, 1, 1, 987, 654, 321, "default largestUnit is hour");
const lambda = earlier.until(later, () => {});
TemporalHelpers.assertDuration(lambda, 0, 0, 0, 0, 25, 1, 1, 987, 654, 321, "default largestUnit is hour");
