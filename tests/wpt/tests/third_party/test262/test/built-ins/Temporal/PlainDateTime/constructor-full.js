// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: Checking an explicitly constructed instance with all arguments
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, "iso8601");

TemporalHelpers.assertPlainDateTime(datetime,
  1976, 11, "M11", 18, 15, 23, 30, 123, 456, 789,
  "check instance (all arguments supplied)"
);

assert.sameValue(
  datetime.calendarId,
  "iso8601",
  "calendar supplied in constructor can be extracted and is unchanged"
);
