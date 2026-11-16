// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.plaindatetimeiso
description: >
  Time zone parsing from ISO strings does not accept sub-minute UTC offset as
  time zone identifier
features: [Temporal]
---*/

for (const timeZone of [
  "-12:12:59.9",
  "-12:12:59",
  "-121259",
  "+15:59:00",
  "-170100",
  "2021-08-19T17:30:45.123456789-12:12:59.9[-12:12:59.9]",
  "2021-08-19T17:30:45-12:12:59[-12:12:59]",
  "2021-08-19T17:30:45-121259[-121259]",
  "2021-08-19T17:30:45+15:59:00[+15:59:00]",
  "2021-08-19T17:30:45-170100[-170100]",
]) {
  assert.throws(
    RangeError,
    () => Temporal.Now.plainDateTimeISO(timeZone),
    `${timeZone} is not a valid time zone string`
  );
}
