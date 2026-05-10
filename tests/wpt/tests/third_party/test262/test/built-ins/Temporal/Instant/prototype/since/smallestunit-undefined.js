// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Fallback value for smallestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.Instant(1_000_000_000_000_000_000n);
const later = new Temporal.Instant(1_000_090_061_987_654_321n);

const explicit = later.since(earlier, { smallestUnit: undefined });
TemporalHelpers.assertDuration(explicit, 0, 0, 0, 0, 0, 0, 90061, 987, 654, 321, "default smallestUnit is nanosecond");
const implicit = later.since(earlier, {});
TemporalHelpers.assertDuration(implicit, 0, 0, 0, 0, 0, 0, 90061, 987, 654, 321, "default smallestUnit is nanosecond");
const lambda = later.since(earlier, () => {});
TemporalHelpers.assertDuration(lambda, 0, 0, 0, 0, 0, 0, 90061, 987, 654, 321, "default smallestUnit is nanosecond");
