// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Checking limits of representable PlainDateTime
features: [Temporal]
---*/

const min = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);
const max = new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999, 999, 999);

["day", "hour", "minute", "second", "millisecond", "microsecond"].forEach((smallestUnit) => {
  assert.throws(
    RangeError,
    () => min.round({ smallestUnit, roundingMode: "floor" }),
    `rounding beyond limit (unit = ${smallestUnit}, rounding mode = floor)`
  );
  assert.throws(
    RangeError,
    () => max.round({ smallestUnit, roundingMode: "ceil" }),
    `rounding beyond limit (unit = ${smallestUnit}, rounding mode = ceil)`
  );
});
