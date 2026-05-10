// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Various string arguments passed to from()
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from("1976-11-18T15:23:30[UTC]").toPlainDateTime(),
  1976, 11, "M11", 18, 15, 23, 30, 0, 0, 0,
  "date and time (no subseconds)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from("1976-11-18T15:23:30.001[UTC]").toPlainDateTime(),
  1976, 11, "M11", 18, 15, 23, 30, 1, 0, 0,
  "date and time (millisecond)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from("1976-11-18T15:23:30.001123[UTC]").toPlainDateTime(),
  1976, 11, "M11", 18, 15, 23, 30, 1, 123, 0,
  "date and time (microsecond)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from("1976-11-18T15:23:30.001123456[UTC]").toPlainDateTime(),
  1976, 11, "M11", 18, 15, 23, 30, 1, 123, 456,
  "date and time (nanosecond)"
);
