// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: An instance of PlainTime can be used as an argument
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dt = new Temporal.PlainDateTime(2015, 12, 7, 3, 24, 30, 0, 3, 500);
const hour = 11;
const minute = 22;
const time = new Temporal.PlainTime(hour, minute);

TemporalHelpers.assertPlainDateTime(
  dt.withPlainTime(time),
  2015,
  12,
  "M12",
  7,
  hour,
  minute,
  0,
  0,
  0,
  0,
  "PlainTime argument works"
);
