// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Type conversions for smallestUnit option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.Instant(1_000_000_000_000_000_000n);
const later = new Temporal.Instant(1_000_090_061_987_654_321n);
TemporalHelpers.checkStringOptionWrongType("smallestUnit", "microsecond",
  (smallestUnit) => earlier.until(later, { smallestUnit }),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 90061, 987, 654, 0, descr),
);
