// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Duration object arguments are handled
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const jan31 = new Temporal.PlainDateTime(2020, 1, 31, 15, 0);

const subtractWithDuration = jan31.subtract(new Temporal.Duration(0, 1, 0, 0, 0, 1));
TemporalHelpers.assertPlainDateTime(
  subtractWithDuration,
  2019, 12, "M12", 31, 14, 59, 0, 0, 0, 0,
  "Duration argument"
);
