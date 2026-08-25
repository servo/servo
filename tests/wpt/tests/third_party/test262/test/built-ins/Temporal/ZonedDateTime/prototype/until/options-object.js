// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Empty or a function object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

const result1 = instance.until(new Temporal.ZonedDateTime(3600_000_000_000n, "UTC"), {});
TemporalHelpers.assertDuration(
  result1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
  "options may be an empty plain object"
);

const result2 = instance.until(new Temporal.ZonedDateTime(3600_000_000_000n, "UTC"), () => {});
TemporalHelpers.assertDuration(
  result2, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
  "options may be a function object"
);
