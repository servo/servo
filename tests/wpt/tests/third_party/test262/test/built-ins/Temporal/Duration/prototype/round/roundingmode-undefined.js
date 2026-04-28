// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Fallback value for roundingMode option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 0, 12, 34, 56, 123, 987, 500);

const explicit1 = duration.round({ smallestUnit: "microsecond", roundingMode: undefined });
TemporalHelpers.assertDuration(explicit1, 0, 0, 0, 0, 12, 34, 56, 123, 988, 0, "default roundingMode is halfExpand");
const implicit1 = duration.round({ smallestUnit: "microsecond" });
TemporalHelpers.assertDuration(implicit1, 0, 0, 0, 0, 12, 34, 56, 123, 988, 0, "default roundingMode is halfExpand");

const explicit2 = duration.round({ smallestUnit: "millisecond", roundingMode: undefined });
TemporalHelpers.assertDuration(explicit2, 0, 0, 0, 0, 12, 34, 56, 124, 0, 0, "default roundingMode is halfExpand");
const implicit2 = duration.round({ smallestUnit: "millisecond" });
TemporalHelpers.assertDuration(implicit2, 0, 0, 0, 0, 12, 34, 56, 124, 0, 0, "default roundingMode is halfExpand");

const explicit3 = duration.round({ smallestUnit: "second", roundingMode: undefined });
TemporalHelpers.assertDuration(explicit3, 0, 0, 0, 0, 12, 34, 56, 0, 0, 0, "default roundingMode is halfExpand");
const implicit3 = duration.round({ smallestUnit: "second" });
TemporalHelpers.assertDuration(implicit3, 0, 0, 0, 0, 12, 34, 56, 0, 0, 0, "default roundingMode is halfExpand");
