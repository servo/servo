// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: >
  If timeZoneName is "auto", the time zone ID should be included and prefixed
  with "!"
features: [Temporal]
---*/

const tests = [
  ["UTC", "1970-01-01T01:01:01.987654321+00:00[!UTC]", "built-in UTC"],
  ["+01:00", "1970-01-01T02:01:01.987654321+01:00[!+01:00]", "built-in offset"],
];

for (const [timeZone, expected, description] of tests) {
  const date = new Temporal.ZonedDateTime(3661_987_654_321n, timeZone);
  const result = date.toString({ timeZoneName: "critical" });
  assert.sameValue(result, expected, `${description} time zone for timeZoneName = critical`);
}
