// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: ZonedDateTime constructor accepts link names as time zone ID input
features: [Temporal, canonical-tz]
---*/

const testCases = [
  "GMT",  // Link    Etc/GMT                         GMT
  "Etc/Universal",  // Link    Etc/UTC                         Etc/Universal
  "Etc/Zulu",  // Link    Etc/UTC                         Etc/Zulu
  "Etc/Greenwich",  // Link    Etc/GMT                         Etc/Greenwich
  "Etc/GMT-0",  // Link    Etc/GMT                         Etc/GMT-0
  "Etc/GMT+0",  // Link    Etc/GMT                         Etc/GMT+0
  "Etc/GMT0",  // Link    Etc/GMT                         Etc/GMT0
];

for (let id of testCases) {
  const instance = new Temporal.ZonedDateTime(0n, id);
  assert.sameValue(instance.timeZoneId, id);
}
