// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: An ISO 8601 string can be converted to a calendar ID in Calendar
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(0n, "UTC");

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
  const arg = { year: 1970, monthCode: "M01", day: 1, timeZone: "UTC", calendar };
  const result1 = Temporal.ZonedDateTime.compare(arg, datetime);
  assert.sameValue(result1, 0, `Calendar created from string "${calendar}" (first argument)`);
  const result2 = Temporal.ZonedDateTime.compare(datetime, arg);
  assert.sameValue(result2, 0, `Calendar created from string "${calendar}" (second argument)`);
}
