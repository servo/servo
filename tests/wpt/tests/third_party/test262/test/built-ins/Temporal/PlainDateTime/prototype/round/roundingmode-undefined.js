// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Fallback value for roundingMode option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 123, 987, 500);

const explicit1 = datetime.round({ smallestUnit: "microsecond", roundingMode: undefined });
TemporalHelpers.assertPlainDateTime(explicit1, 2000, 5, "M05", 2, 12, 34, 56, 123, 988, 0, "default roundingMode is halfExpand");
const implicit1 = datetime.round({ smallestUnit: "microsecond" });
TemporalHelpers.assertPlainDateTime(implicit1, 2000, 5, "M05", 2, 12, 34, 56, 123, 988, 0, "default roundingMode is halfExpand");

const explicit2 = datetime.round({ smallestUnit: "millisecond", roundingMode: undefined });
TemporalHelpers.assertPlainDateTime(explicit2, 2000, 5, "M05", 2, 12, 34, 56, 124, 0, 0, "default roundingMode is halfExpand");
const implicit2 = datetime.round({ smallestUnit: "millisecond" });
TemporalHelpers.assertPlainDateTime(implicit2, 2000, 5, "M05", 2, 12, 34, 56, 124, 0, 0, "default roundingMode is halfExpand");

const explicit3 = datetime.round({ smallestUnit: "second", roundingMode: undefined });
TemporalHelpers.assertPlainDateTime(explicit3, 2000, 5, "M05", 2, 12, 34, 56, 0, 0, 0, "default roundingMode is halfExpand");
const implicit3 = datetime.round({ smallestUnit: "second" });
TemporalHelpers.assertPlainDateTime(implicit3, 2000, 5, "M05", 2, 12, 34, 56, 0, 0, 0, "default roundingMode is halfExpand");
