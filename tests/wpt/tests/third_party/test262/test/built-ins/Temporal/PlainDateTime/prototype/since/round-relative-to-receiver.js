// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Values are rounded relative to the receiver
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(2019, 1, 1);
const dt2 = new Temporal.PlainDateTime(2020, 7, 2);

TemporalHelpers.assertDuration(
  dt2.since(dt1, { smallestUnit: "years", roundingMode: "halfExpand" }),
  1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "rounds relative to the receiver (positive case)"
);

TemporalHelpers.assertDuration(
  dt1.since(dt2, { smallestUnit: "years", roundingMode: "halfExpand" }),
  -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "rounds relative to the receiver (negative case)"
);
