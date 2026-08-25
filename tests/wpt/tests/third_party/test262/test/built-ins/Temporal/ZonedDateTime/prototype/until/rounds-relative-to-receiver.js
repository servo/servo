// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Rounds relative to the receiver.
includes: [temporalHelpers.js]
features: [Temporal]
---*/


// rounds relative to the receiver
const dt1 = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
const dt2 = Temporal.ZonedDateTime.from("2020-07-02T00:00+00:00[UTC]");
TemporalHelpers.assertDuration(dt1.until(dt2, {
  smallestUnit: "years",
  roundingMode: "halfExpand"
}), 2, 0, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(dt2.until(dt1, {
  smallestUnit: "years",
  roundingMode: "halfExpand"
}), -1, 0, 0, 0, 0, 0, 0, 0, 0, 0);

