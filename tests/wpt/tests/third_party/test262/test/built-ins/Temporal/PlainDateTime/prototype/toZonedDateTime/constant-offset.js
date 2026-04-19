// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Result in time zone with constant UTC offset
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2019, 2, 16, 23, 45);
["earlier", "later", "compatible", "reject"].forEach((disambiguation) => {
  const result = instance.toZonedDateTime("+03:30", { disambiguation });
  assert.sameValue(result.epochNanoseconds, 1550348100_000_000_000n, "Result is 2019-02-16T20:15Z regardless of disambiguation");
});
