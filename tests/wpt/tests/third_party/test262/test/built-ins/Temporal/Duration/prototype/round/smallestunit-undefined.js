// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Fallback value for smallestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 0, 1, 2, 3, 123, 456, 789);
const explicit1 = duration.round({ largestUnit: "day", smallestUnit: undefined });
TemporalHelpers.assertDuration(explicit1, 0, 0, 0, 0, 1, 2, 3, 123, 456, 789, "default smallestUnit is nanosecond");
const implicit1 = duration.round({ largestUnit: "day" });
TemporalHelpers.assertDuration(implicit1, 0, 0, 0, 0, 1, 2, 3, 123, 456, 789, "default smallestUnit is nanosecond");
