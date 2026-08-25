// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Truncation (trunc) is the default rounding mode
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2019, 1, 8, 8, 22, 36, 123, 456, 789);
const later = new Temporal.PlainDateTime(2021, 9, 7, 12, 39, 40, 987, 654, 321);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes" }),
  0, 0, 0, 973, 4, 17, 0, 0, 0, 0,
  "trunc is the default (round up)"
);

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "seconds" }),
  0, 0, 0, 973, 4, 17, 4, 0, 0, 0,
  "trunc is the default (round down)"
);
