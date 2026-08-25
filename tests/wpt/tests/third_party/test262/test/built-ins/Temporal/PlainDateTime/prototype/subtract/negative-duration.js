// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Negative durations can be supplied
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const jan31 = new Temporal.PlainDateTime(2020, 1, 31, 15, 0);

TemporalHelpers.assertPlainDateTime(
  jan31.subtract({ minutes: -30 }),
  2020, 1, "M01", 31, 15, 30, 0, 0, 0, 0,
  "negative minutes"
);

TemporalHelpers.assertPlainDateTime(
  jan31.subtract({ seconds: -30 }),
  2020, 1, "M01", 31, 15, 0, 30, 0, 0, 0,
  "negative seconds"
);
