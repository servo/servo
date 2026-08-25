// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Type conversions for smallestUnit option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 0, 12, 34, 56, 123, 987, 500);
TemporalHelpers.checkStringOptionWrongType("smallestUnit", "microsecond",
  (smallestUnit) => duration.round({ smallestUnit }),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 12, 34, 56, 123, 988, 0, descr),
);
