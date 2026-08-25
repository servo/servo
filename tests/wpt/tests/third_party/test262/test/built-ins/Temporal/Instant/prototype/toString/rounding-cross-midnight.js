// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Rounding can cross midnight
features: [Temporal]
---*/

const instant = new Temporal.Instant(946_684_799_999_999_999n);  // one nanosecond before 2000-01-01T00:00:00
for (const roundingMode of ["ceil", "halfExpand"]) {
  assert.sameValue(instant.toString({ fractionalSecondDigits: 8, roundingMode }), "2000-01-01T00:00:00.00000000Z");
}
