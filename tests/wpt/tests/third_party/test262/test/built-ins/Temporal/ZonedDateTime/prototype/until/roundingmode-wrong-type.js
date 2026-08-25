// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Type conversions for roundingMode option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");
const later = new Temporal.ZonedDateTime(1_000_090_061_123_987_500n, "UTC");
TemporalHelpers.checkStringOptionWrongType("roundingMode", "trunc",
  (roundingMode) => earlier.until(later, { smallestUnit: "microsecond", roundingMode }),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 25, 1, 1, 123, 987, 0, descr),
);
