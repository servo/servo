// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Fallback value for largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration1 = new Temporal.Duration(0, 0, 0, 0, 1, 120, 1, 123, 456, 789);
const explicit1 = duration1.round({ largestUnit: undefined, smallestUnit: "nanosecond" });
TemporalHelpers.assertDuration(explicit1, 0, 0, 0, 0, 3, 0, 1, 123, 456, 789, "default largestUnit is largest in input");
const implicit1 = duration1.round({ smallestUnit: "nanosecond" });
TemporalHelpers.assertDuration(implicit1, 0, 0, 0, 0, 3, 0, 1, 123, 456, 789, "default largestUnit is largest in input");

const duration2 = new Temporal.Duration(0, 0, 0, 0, 0, 120, 1, 123, 456, 789);
const explicit2 = duration2.round({ largestUnit: undefined, smallestUnit: "nanosecond" });
TemporalHelpers.assertDuration(explicit2, 0, 0, 0, 0, 0, 120, 1, 123, 456, 789, "default largestUnit is largest in input");
const implicit2 = duration2.round({ smallestUnit: "nanosecond" });
TemporalHelpers.assertDuration(implicit2, 0, 0, 0, 0, 0, 120, 1, 123, 456, 789, "default largestUnit is largest in input");
