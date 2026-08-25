// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Assumes a different default for largestUnit if smallestUnit is larger than seconds.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.Instant.from("1976-11-18T15:23:30.123456789Z");
const later = Temporal.Instant.from("2019-10-29T10:46:38.271986102Z");

TemporalHelpers.assertDuration(later.since(earlier, {
  smallestUnit: "hours",
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 376435, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(later.since(earlier, {
  smallestUnit: "minutes",
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 0, 22586123, 0, 0, 0, 0);
