// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: Type conversions for roundingMode option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_123_987_500n);
TemporalHelpers.checkStringOptionWrongType("roundingMode", "halfExpand",
  (roundingMode) => instant.round({ smallestUnit: "microsecond", roundingMode }),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_000_000_123_988_000n, descr),
);
