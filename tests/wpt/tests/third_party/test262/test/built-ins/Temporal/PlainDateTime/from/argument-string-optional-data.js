// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Some parts of a string argument may be omitted
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30+00"),
  1976, 11, "M11", 18, 15, 23, 30, 0, 0, 0,
  "optional parts (no minute after offset)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15"),
  1976, 11, "M11", 18, 15, 0, 0, 0, 0, 0,
  "optional parts (no minute in time part)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18"),
  1976, 11, "M11", 18, 0, 0, 0, 0, 0, 0,
  "optional parts (no time part)"
);
