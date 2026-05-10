// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: String arguments are acceptable
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30"),
  1976, 11, "M11", 18, 15, 23, 30, 0, 0, 0,
  "date and time (no subseconds)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.001"),
  1976, 11, "M11", 18, 15, 23, 30, 1, 0, 0,
  "date and time (millisecond)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.001123"),
  1976, 11, "M11", 18, 15, 23, 30, 1, 123, 0,
  "date and time (microsecond)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.001123456"),
  1976, 11, "M11", 18, 15, 23, 30, 1, 123, 456,
  "date and time (nanosecond)"
);
