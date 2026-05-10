// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Type conversions for roundingMode option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_123_987_500n, "UTC");
TemporalHelpers.checkStringOptionWrongType("roundingMode", "halfExpand",
  (roundingMode) => datetime.round({ smallestUnit: "microsecond", roundingMode }),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_000_000_123_988_000n, descr),
);
