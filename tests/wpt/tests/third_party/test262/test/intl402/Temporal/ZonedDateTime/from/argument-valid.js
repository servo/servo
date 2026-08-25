// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Built-in time zones are parsed correctly out of valid strings
features: [Temporal, canonical-tz]
---*/

const valids = [
  ["Africa/Bissau"],
  ["America/Belem"],
  ["Europe/Vienna"],
  ["America/New_York"],
  ["Africa/CAIRO", "Africa/Cairo"],
  ["Asia/Ulan_Bator"],
  ["GMT"],
  ["etc/gmt", "Etc/GMT"],
  ["1994-11-05T08:15:30-05:00[America/New_York]", "America/New_York"],
  ["1994-11-05T08:15:30-05[America/New_York]", "America/New_York"],
];

for (const [valid, canonical = valid] of valids) {
  const result = Temporal.ZonedDateTime.from({ year: 1970, month: 1, day: 1, timeZone: valid });
  assert.sameValue(Object.getPrototypeOf(result), Temporal.ZonedDateTime.prototype);
  assert.sameValue(result.timeZoneId, canonical);
}
