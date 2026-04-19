// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: Rounding can cross midnight
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(23, 59, 59, 999, 999, 999);  // one nanosecond before 00:00:00
for (const roundingMode of ["ceil", "halfExpand"]) {
  assert.sameValue(plainTime.toString({ fractionalSecondDigits: 8, roundingMode }), "00:00:00.00000000");
}
