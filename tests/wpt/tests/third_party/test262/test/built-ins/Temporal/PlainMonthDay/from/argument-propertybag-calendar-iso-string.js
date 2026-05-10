// Copyright (C) 2025 Brage Hogstad, University of Bergen. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: An ISO 8601 string can be converted to a calendar ID in Calendar
includes: [temporalHelpers.js]
features: [Temporal]
---*/

for (const calendar of [
  "2020-01-01",
  "2020-01-01[u-ca=iso8601]",
  "2020-01-01T00:00:00.000000000",
  "2020-01-01T00:00:00.000000000[u-ca=iso8601]",
  "01-01",
  "01-01[u-ca=iso8601]",
  "2020-01",
  "2020-01[u-ca=iso8601]",
]) {
  const arg = { monthCode: "M11", day: 18, calendar };
  const result = Temporal.PlainMonthDay.from(arg);
  TemporalHelpers.assertPlainMonthDay(result, "M11", 18, `Calendar created from string "${calendar}"`);
  assert.sameValue(result.calendarId, "iso8601", "calendar string is iso8601");
}
