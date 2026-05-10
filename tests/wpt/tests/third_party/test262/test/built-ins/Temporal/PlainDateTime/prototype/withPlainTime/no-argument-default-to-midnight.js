// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: If no argument is given, default to midnight
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const dt = new Temporal.PlainDateTime(2015, 12, 7, 3, 24, 30, 0, 3, 500);

TemporalHelpers.assertPlainDateTime(
  dt.withPlainTime(),
  2015, 12, "M12", 7, 0, 0, 0, 0, 0, 0,
  "no argument defaults to midnight"
);
