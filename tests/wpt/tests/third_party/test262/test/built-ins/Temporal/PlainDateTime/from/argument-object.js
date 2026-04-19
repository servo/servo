// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Plain objects are acceptable
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({year: 1976, month: 11, monthCode: "M11", day: 18}),
  1976, 11, "M11", 18, 0, 0, 0, 0, 0, 0,
  "plain object with month & month code"
);

assert.throws(
  TypeError,
  () => Temporal.PlainDateTime.from({}),
  "empty object throws"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({year: 1976, month: 11, day: 18, millisecond: 123}),
  1976, 11, "M11", 18, 0, 0, 0, 123, 0, 0,
  "plain object with month but not month code"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({year: 1976, monthCode: "M09", day: 18, millisecond: 123}),
  1976, 9, "M09", 18, 0, 0, 0, 123, 0, 0,
  "plain object with month code but not month"
);


TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({year: 1976, month: 11, day: 18, hours: 12}),
  1976, 11, "M11", 18, 0, 0, 0, 0, 0, 0,
  "incorrectly-spelled properties (e.g., plural \"hours\") are ignored"
);
