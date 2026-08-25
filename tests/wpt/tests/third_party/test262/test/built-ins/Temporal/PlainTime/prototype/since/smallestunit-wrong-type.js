// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: Type conversions for smallestUnit option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainTime(12, 34, 56, 0, 0, 0);
const later = new Temporal.PlainTime(13, 35, 57, 987, 654, 321);
TemporalHelpers.checkStringOptionWrongType("smallestUnit", "microsecond",
  (smallestUnit) => later.since(earlier, { smallestUnit }),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 1, 1, 1, 987, 654, 0, descr),
);
