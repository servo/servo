// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Time zone IDs are valid input for a time zone
features: [Temporal]
---*/

// The following are all valid strings so should not throw:

["UTC", "+01:00"].forEach((timeZone) => {
  Temporal.Duration.compare(new Temporal.Duration(1), new Temporal.Duration(), { relativeTo: { year: 2000, month: 5, day: 2, timeZone } });
});
