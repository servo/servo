// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: Type conversions for roundingMode option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 123, 987, 500);
TemporalHelpers.checkStringOptionWrongType("roundingMode", "halfExpand",
  (roundingMode) => time.round({ smallestUnit: "microsecond", roundingMode }),
  (result, descr) => TemporalHelpers.assertPlainTime(result, 12, 34, 56, 123, 988, 0, descr),
);
