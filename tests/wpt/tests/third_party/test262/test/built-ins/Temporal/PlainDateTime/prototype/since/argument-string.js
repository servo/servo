// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Date-like string arguments are acceptable
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

TemporalHelpers.assertDuration(
  dt.since("2019-10-29T10:46:38.271986102"),
  0, 0, 0, -15684, -19, -23, -8, -148, -529, -313,
  "casts argument (string)"
);
