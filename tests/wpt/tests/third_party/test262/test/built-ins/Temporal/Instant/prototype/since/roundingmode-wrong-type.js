// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Type conversions for roundingMode option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.Instant(1_000_000_000_000_000_000n);
const later = new Temporal.Instant(1_000_090_061_123_987_500n);
TemporalHelpers.checkStringOptionWrongType("roundingMode", "trunc",
  (roundingMode) => later.since(earlier, { smallestUnit: "microsecond", roundingMode }),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 90061, 123, 987, 0, descr),
);
