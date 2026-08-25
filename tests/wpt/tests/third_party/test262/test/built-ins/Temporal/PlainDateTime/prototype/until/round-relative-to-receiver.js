// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Rounding happens relative to receiver
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(2019, 1, 1);
const dt2 = new Temporal.PlainDateTime(2020, 7, 2);
const options = { smallestUnit: "years", roundingMode: "halfExpand" };

TemporalHelpers.assertDuration(
  dt1.until(dt2, options),
  2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "rounds relative to the receiver (positive case)"
);

TemporalHelpers.assertDuration(
  dt2.until(dt1, options),
  -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "rounds relative to the receiver (negative case)"
);
