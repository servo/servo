// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: RangeError thrown when roundingMode option not one of the allowed string values
features: [Temporal]
---*/

const earlier = new Temporal.PlainDate(2000, 5, 2);
const later = new Temporal.PlainDate(2001, 6, 3);
for (const roundingMode of ["other string", "cile", "CEIL", "ce\u0131l", "auto", "halfexpand", "floor\0"]) {
  assert.throws(RangeError, () => earlier.until(later, { smallestUnit: "microsecond", roundingMode }));
}
