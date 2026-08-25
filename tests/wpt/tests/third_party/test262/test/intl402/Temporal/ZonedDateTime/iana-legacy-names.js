// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: IANA legacy names must be supported
features: [Temporal, canonical-tz]
---*/

const legacyNames = [
  "Etc/GMT0",
  "GMT0",
  "GMT-0",
  "GMT+0",
  "EST5EDT", 
  "CST6CDT", 
  "MST7MDT", 
  "PST8PDT"
];

legacyNames.forEach((arg) => {
  const instance = new Temporal.ZonedDateTime(0n, arg);
  assert.sameValue(instance.timeZoneId, arg, `"${arg}" does not match "${instance.timeZoneId}" time zone identifier`);
});
