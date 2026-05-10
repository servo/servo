// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: RangeError thrown when smallestUnit is larger than largestUnit
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const relativeTo = Temporal.PlainDate.from('2020-01-01');
const units = ["years", "months", "weeks", "days", "hours", "minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"];
for (let largestIdx = 1; largestIdx < units.length; largestIdx++) {
  for (let smallestIdx = 0; smallestIdx < largestIdx; smallestIdx++) {
    const largestUnit = units[largestIdx];
    const smallestUnit = units[smallestIdx];
    assert.throws(RangeError, () => d.round({ largestUnit, smallestUnit, relativeTo }),
        `${smallestUnit} > ${largestUnit}`);
  }
}
