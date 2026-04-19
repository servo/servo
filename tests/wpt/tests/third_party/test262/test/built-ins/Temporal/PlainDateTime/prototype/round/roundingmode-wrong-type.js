// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Type conversions for roundingMode option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 123, 987, 500);
TemporalHelpers.checkStringOptionWrongType("roundingMode", "halfExpand",
  (roundingMode) => datetime.round({ smallestUnit: "microsecond", roundingMode }),
  (result, descr) => TemporalHelpers.assertPlainDateTime(result, 2000, 5, "M05", 2, 12, 34, 56, 123, 988, 0, descr),
);
