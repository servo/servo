// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Duration object arguments are handled
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const jan31 = new Temporal.PlainDateTime(2020, 1, 31, 15, 0);

TemporalHelpers.assertPlainDateTime(
  jan31.add(Temporal.Duration.from("P1MT1S")),
  2020, 2, "M02", 29, 15, 0, 1, 0, 0, 0,
  "Duration argument"
);
