// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Time zone IDs are valid input for a time zone
features: [Temporal]
---*/

["UTC", "+01:30"].forEach((timeZone) => {
  const epoch = new Temporal.ZonedDateTime(0n, timeZone);

  // These should be valid input and not throw
  Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, epoch);
  Temporal.ZonedDateTime.compare(epoch, { year: 2020, month: 5, day: 2, timeZone });
});
