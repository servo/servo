// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: ZonedDateTime constructor accepts link names as time zone ID input
features: [Temporal, canonical-tz]
---*/

const testCases = [
  "Africa/Asmera",  // Link Africa/Asmara Africa/Asmera
  "America/Kralendijk",  // Link    America/Curacao America/Kralendijk
  "America/Lower_Princes",  // Link    America/Curacao America/Lower_Princes
  "America/Marigot",  // Link America/Port_of_Spain America/Marigot
  "America/St_Barthelemy",  // Link America/Port_of_Spain America/St_Barthelemy
  "America/Virgin",  // Link America/St_Thomas America/Virgin
  "Antarctica/South_Pole",  // Link Antarctica/McMurdo Antarctica/South_Pole
  "Asia/Chungking",  // Link Asia/Chongqing Asia/Chungking
];

for (let id of testCases) {
  const instance = new Temporal.ZonedDateTime(0n, id);
  assert.sameValue(instance.timeZoneId, id);
}
