// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withtimezone
description: Time zone IDs are valid input for a time zone
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

["UTC", "+01:30"].forEach((timeZone) => {
  const result = instance.withTimeZone(timeZone);
  assert.sameValue(result.timeZoneId, timeZone, `time zone slot should store string "${timeZone}"`);
});
