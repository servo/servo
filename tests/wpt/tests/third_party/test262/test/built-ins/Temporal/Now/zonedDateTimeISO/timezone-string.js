// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.zoneddatetimeiso
description: Time zone IDs are valid input for a time zone
features: [Temporal]
---*/

["UTC", "+01:30"].forEach((timeZone) => {
  const result = Temporal.Now.zonedDateTimeISO(timeZone);
  assert.sameValue(result.timeZoneId, timeZone, `Time zone created from string "${timeZone}"`);
});
