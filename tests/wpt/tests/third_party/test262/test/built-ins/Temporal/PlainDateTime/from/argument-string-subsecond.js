// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Up to nine digits of sub-second precision are acceptable
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.1"),
  1976, 11, "M11", 18, 15, 23, 30, 100, 0, 0,
  "various precisions are possible (one decimal digit)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.12"),
  1976, 11, "M11", 18, 15, 23, 30, 120, 0, 0,
  "various precisions are possible (two decimal digits)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.123"),
  1976, 11, "M11", 18, 15, 23, 30, 123, 0, 0,
  "various precisions are possible (three decimal digits)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.1234"),
  1976, 11, "M11", 18, 15, 23, 30, 123, 400, 0,
  "various precisions are possible (four decimal digits)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.12345"),
  1976, 11, "M11", 18, 15, 23, 30, 123, 450, 0,
  "various precisions are possible (five decimal digits)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.123456"),
  1976, 11, "M11", 18, 15, 23, 30, 123, 456, 0,
  "various precisions are possible (six decimal digits)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.1234567"),
  1976, 11, "M11", 18, 15, 23, 30, 123, 456, 700,
  "various precisions are possible (seven decimal digits)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.12345678"),
  1976, 11, "M11", 18, 15, 23, 30, 123, 456, 780,
  "various precisions are possible (eight decimal digits)"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from("1976-11-18T15:23:30.123456789"),
  1976, 11, "M11", 18, 15, 23, 30, 123, 456, 789,
  "various precisions are possible (nine decimal digits)"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from("1976-11-18T15:23:30.1234567891"),
  "ten decimal digits is too much"
);
