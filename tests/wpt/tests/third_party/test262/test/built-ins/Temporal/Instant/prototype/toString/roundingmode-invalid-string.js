// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: RangeError thrown when roundingMode option not one of the allowed string values
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_123_987_500n);
for (const roundingMode of ["other string", "cile", "CEIL", "ce\u0131l", "auto", "halfexpand", "floor\0"]) {
  assert.throws(RangeError, () => instant.toString({ smallestUnit: "microsecond", roundingMode }));
}
