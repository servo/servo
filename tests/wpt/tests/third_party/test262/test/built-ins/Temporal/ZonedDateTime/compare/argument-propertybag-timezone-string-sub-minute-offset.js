// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: >
  Time zone parsing from ISO strings does not accept sub-minute UTC offset as
  time zone identifier
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1588371240_000_000_000n, "+01:46");

for (const timeZone of ["-12:12:59.9", "2021-08-19T17:30:45.123456789-12:12:59.9[-12:12:59.9]"]) {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, instance),
    `${timeZone} is not a valid time zone string (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.compare(instance, { year: 2020, month: 5, day: 2, timeZone }),
    `${timeZone} is not a valid time zone string (second argument)`
  );
}
