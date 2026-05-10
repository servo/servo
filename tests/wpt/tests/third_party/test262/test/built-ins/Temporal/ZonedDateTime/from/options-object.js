// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.from
description: Empty object may be used as options
features: [Temporal]
---*/

assert.sameValue(
  Temporal.ZonedDateTime.from({ year: 1976, month: 11, day: 18, timeZone: "UTC" }, {}).epochNanoseconds, 217123200000000000n, "UTC",
  "options may be an empty plain object"
);

assert.sameValue(
  Temporal.ZonedDateTime.from({ year: 1976, month: 11, day: 18, timeZone: "UTC" }, () => {}).epochNanoseconds, 217123200000000000n, "UTC",
  "options may be an empty function object"
);
