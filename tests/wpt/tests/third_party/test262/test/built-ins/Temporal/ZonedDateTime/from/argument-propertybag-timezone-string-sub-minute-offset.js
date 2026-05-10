// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Time zone parsing from ISO strings does not accept sub-minute UTC offset as
  time zone identifier
features: [Temporal]
---*/

for (const timeZone of ["-12:12:59.9", "2021-08-19T17:30:45.123456789-12:12:59.9[-12:12:59.9]"]) {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.from({ year: 2020, month: 5, day: 2, timeZone }),
    `${timeZone} is not a valid time zone string`
  );
}
