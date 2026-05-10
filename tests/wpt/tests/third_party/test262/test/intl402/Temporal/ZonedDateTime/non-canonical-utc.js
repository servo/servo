// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: ZonedDateTime constructor accepts link names as time zone ID input
features: [Temporal, canonical-tz]
---*/

const testCases = [
  "Etc/GMT",
  "Etc/GMT+0",
  "Etc/GMT-0",
  "Etc/GMT0",
  "Etc/Greenwich",
  "Etc/UCT",
  "Etc/UTC",
  "Etc/Universal",
  "Etc/Zulu",
];

for (let id of testCases) {
  let instance = new Temporal.ZonedDateTime(0n, id);

  assert.sameValue(instance.timeZoneId, id);
}
