// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Rounding balances to the next smallest unit
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 23, 59, 59, 999, 999, 999);

["day", "hour", "minute", "second", "millisecond", "microsecond"].forEach((smallestUnit) => {
  TemporalHelpers.assertPlainDateTime(
    dt.round({ smallestUnit }),
    1976, 11, "M11", 19, 0, 0, 0, 0, 0, 0,
    `balances to next ${smallestUnit}`
  );
});
