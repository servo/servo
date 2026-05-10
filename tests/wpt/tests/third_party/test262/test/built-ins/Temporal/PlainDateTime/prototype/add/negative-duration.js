// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Negative durations can be supplied
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const jan31 = new Temporal.PlainDateTime(2020, 1, 31, 15, 0);

TemporalHelpers.assertPlainDateTime(
  jan31.add({ minutes: -30 }),
  2020, 1, "M01", 31, 14, 30, 0, 0, 0, 0,
  "negative minutes"
);

TemporalHelpers.assertPlainDateTime(
  jan31.add({ seconds: -30 }),
  2020, 1, "M01", 31, 14, 59, 30, 0, 0, 0,
  "negative seconds"
);
