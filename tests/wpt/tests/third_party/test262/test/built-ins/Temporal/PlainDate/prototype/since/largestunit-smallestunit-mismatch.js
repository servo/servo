// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: RangeError thrown when smallestUnit is larger than largestUnit
features: [Temporal]
---*/

const earlier = new Temporal.PlainDate(2000, 5, 2);
const later = new Temporal.PlainDate(2001, 6, 3);
const units = ["years", "months", "weeks", "days"];
for (let largestIdx = 1; largestIdx < units.length; largestIdx++) {
  for (let smallestIdx = 0; smallestIdx < largestIdx; smallestIdx++) {
    const largestUnit = units[largestIdx];
    const smallestUnit = units[smallestIdx];
    assert.throws(RangeError, () => later.since(earlier, { largestUnit, smallestUnit }));
  }
}
