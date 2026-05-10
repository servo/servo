// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Rounds relative to the receiver.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// rounds relative to the receiver
/*
const dt1 = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
const dt2 = Temporal.ZonedDateTime.from("2020-07-02T00:00+00:00[UTC]");
*/
const dt1 = new Temporal.ZonedDateTime(1546300800000000000n, "UTC");
const dt2 = new Temporal.ZonedDateTime(1593648000000000000n, "UTC");

TemporalHelpers.assertDuration(dt2.since(dt1, {
  smallestUnit: "years",
  roundingMode: "halfExpand"
}), 1, 0, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(dt1.since(dt2, {
  smallestUnit: "years",
  roundingMode: "halfExpand"
}), -2, 0, 0, 0, 0, 0, 0, 0, 0, 0);
