// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializedatetimeformat
description: Time zone identifiers are not canonicalized before storing in internal slots
features: [canonical-tz]
---*/

const baseOptions = {
  timeZoneName: "long",
  year: "numeric",
  month: "long",
  day: "numeric",
  hour: "numeric",
  minute: "numeric"
};
const dtf1 = new Intl.DateTimeFormat("en", { ...baseOptions, timeZone: "Asia/Calcutta" });
const dtf2 = new Intl.DateTimeFormat("en", { ...baseOptions, timeZone: "Asia/Kolkata" });

const resolvedId1 = dtf1.resolvedOptions().timeZone;
const resolvedId2 = dtf2.resolvedOptions().timeZone;

const output1 = dtf1.format(0);
const output2 = dtf2.format(0);

assert.sameValue(output1, output2);
assert.sameValue(resolvedId1, "Asia/Calcutta");
assert.sameValue(resolvedId2, "Asia/Kolkata");
