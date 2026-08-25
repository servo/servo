// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Verify that undefined options are handled correctly.
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const jan31 = new Temporal.PlainDateTime(2020, 1, 31, 15, 0);

TemporalHelpers.assertPlainDateTime(
  jan31.subtract({ months: 2 }, {}),
  2019, 11, "M11", 30, 15, 0, 0, 0, 0, 0,
  "options may be empty object"
);

TemporalHelpers.assertPlainDateTime(
  jan31.subtract({ months: 2 }, () => {}),
  2019, 11, "M11", 30, 15, 0, 0, 0, 0, 0,
  "options may be function object"
);
