// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Assumes a different default for largestUnit if smallestUnit is larger than hours.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.ZonedDateTime.from('2019-01-08T09:22:36.123456789+01:00[+01:00]');
const later = Temporal.ZonedDateTime.from('2021-09-07T13:39:40.987654321+01:00[+01:00]');

TemporalHelpers.assertDuration(earlier.until(later, {
  smallestUnit: "years",
  roundingMode: "halfExpand"
}), 3, 0, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(earlier.until(later, {
  smallestUnit: "months",
  roundingMode: "halfExpand"
}), 0, 32, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(earlier.until(later, {
  smallestUnit: "weeks",
  roundingMode: "halfExpand"
}), 0, 0, 139, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(earlier.until(later, {
  smallestUnit: "days",
  roundingMode: "halfExpand"
}), 0, 0, 0, 973, 0, 0, 0, 0, 0, 0);
