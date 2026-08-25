// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: String argument without ISO 8601 time designator "T" allowed
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dt = new Temporal.PlainDateTime(2015, 12, 7, 3, 24, 30, 0, 3, 500);

TemporalHelpers.assertPlainDateTime(
  dt.withPlainTime("12:34"),
  2015, 12, "M12", 7, 12, 34, 0, 0, 0, 0,
  "time-like string works"
);
