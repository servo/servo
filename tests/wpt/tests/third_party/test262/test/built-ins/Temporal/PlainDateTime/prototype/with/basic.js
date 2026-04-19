// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: A variety of "normal" (non-throwing, non-boundary case, non-null, etc.) arguments
esid: sec-temporal.plaindatetime.prototype.with
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ year: 2019 }),
  2019, 11, "M11", 18, 15, 23, 30, 123, 456, 789,
  "with year works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ month: 5 }),
  1976, 5, "M05", 18, 15, 23, 30, 123, 456, 789,
  "with month works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ monthCode: "M05" }),
  1976, 5, "M05", 18, 15, 23, 30, 123, 456, 789,
  "with month code works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ day: 5 }),
  1976, 11, "M11", 5, 15, 23, 30, 123, 456, 789,
  "with day works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ hour: 5 }),
  1976, 11, "M11", 18, 5, 23, 30, 123, 456, 789,
  "with hour works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ minute: 5 }),
  1976, 11, "M11", 18, 15, 5, 30, 123, 456, 789,
  "with minute works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ second: 5 }),
  1976, 11, "M11", 18, 15, 23, 5, 123, 456, 789,
  "with second works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ millisecond: 5 }),
  1976, 11, "M11", 18, 15, 23, 30, 5, 456, 789,
  "with millisecond works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ microsecond: 5 }),
  1976, 11, "M11", 18, 15, 23, 30, 123, 5, 789,
  "with microsecond works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ nanosecond: 5 }),
  1976, 11, "M11", 18, 15, 23, 30, 123, 456, 5,
  "with nanosecond works"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ month: 5, second: 15 }),
  1976, 5, "M05", 18, 15, 23, 15, 123, 456, 789,
  "with month and second works"
);
