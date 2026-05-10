// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: >
  Time zone parsing from ISO strings does not accept sub-minute UTC offset as
  time zone identifier
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(1970, 1, 1);

for (const timeZone of ["-12:12:59.9", "2021-08-19T17:30:45.123456789-12:12:59.9[-12:12:59.9]"]) {
  assert.throws(
    RangeError,
    () => instance.toZonedDateTime(timeZone),
    `${timeZone} is not a valid time zone string`
  );
}
