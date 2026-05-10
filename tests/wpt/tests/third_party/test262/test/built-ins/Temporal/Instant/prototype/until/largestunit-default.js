// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Assumes a different default for largestUnit if smallestUnit is larger than seconds.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.Instant.from("1969-07-24T16:50:35.123456789Z");
const later = Temporal.Instant.from("2019-10-29T10:46:38.271986102Z");

TemporalHelpers.assertDuration(earlier.until(later, {
  smallestUnit: "hours",
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 440610, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(earlier.until(later, {
  smallestUnit: "minutes",
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 0, 26436596, 0, 0, 0, 0);
