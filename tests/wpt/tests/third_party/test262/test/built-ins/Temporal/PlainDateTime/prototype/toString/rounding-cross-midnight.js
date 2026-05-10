// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Rounding can cross midnight
features: [Temporal]
---*/

const plainDateTime = new Temporal.PlainDateTime(1999, 12, 31, 23, 59, 59, 999, 999, 999);  // one nanosecond before 2000-01-01T00:00:00
for (const roundingMode of ["ceil", "halfExpand"]) {
  assert.sameValue(plainDateTime.toString({ fractionalSecondDigits: 8, roundingMode }), "2000-01-01T00:00:00.00000000");
}
