// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Empty or a function object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

const result1 = instance.since(new Temporal.Instant(3600_000_000_000n), {});
TemporalHelpers.assertDuration(
  result1, 0, 0, 0, 0, 0, 0, -3600, 0, 0, 0,
  "options may be an empty plain object"
);

const result2 = instance.since(new Temporal.Instant(3600_000_000_000n), () => {});
TemporalHelpers.assertDuration(
  result2, 0, 0, 0, 0, 0, 0, -3600, 0, 0, 0,
  "options may be a function object"
);
