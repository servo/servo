// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Type conversions for largestUnit option
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");
const later = new Temporal.ZonedDateTime(1_000_090_061_987_654_321n, "UTC");
TemporalHelpers.checkStringOptionWrongType("largestUnit", "year",
  (largestUnit) => later.since(earlier, { largestUnit }),
  (result, descr) => TemporalHelpers.assertDuration(result, 0, 0, 0, 1, 1, 1, 1, 987, 654, 321, descr),
);
